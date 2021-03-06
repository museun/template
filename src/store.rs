use crate::{Error, LoadFunction, TemplateMap};

use std::path::PathBuf;
use std::time::SystemTime;

/// A backing store for a set of templates
pub trait TemplateStore {
    /// Tries to parse the template map
    ///
    /// # Errors
    /// - Any I/O error associated with fetching this data
    /// - Any deserialization error
    // TODO make this return an Result<Status, Error>
    fn parse_map(&mut self) -> Result<TemplateMap<String>, Error>;
    /// Returns whether the template changed
    fn changed(&mut self) -> bool;
}

/// A file-based backing for templates
pub struct FileStore {
    file: PathBuf,
    last: Option<SystemTime>,
    loader: LoadFunction,
}

impl std::fmt::Debug for FileStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileStore")
            .field("file", &self.file)
            .field("last", &self.last)
            .finish()
    }
}

impl FileStore {
    /// Create a store from this `PathBuf`
    ///
    /// # Errors
    /// - File wasn't found / not readable
    pub fn new(file: PathBuf, loader: LoadFunction) -> Result<Self, Error> {
        Ok(Self {
            file,
            last: None,
            loader,
        })
    }
}

impl TemplateStore for FileStore {
    fn parse_map(&mut self) -> Result<TemplateMap<String>, Error> {
        (self.loader)(&std::fs::read_to_string(&self.file)?)
    }

    fn changed(&mut self) -> bool {
        if self.last.is_none() {
            log::debug!("FileStore initial changed");
            self.last.replace(std::time::SystemTime::now());
            return true;
        }

        // TODO clean this up (this breaks the Option<T: TemplateStore>)
        match std::fs::metadata(&self.file)
            .and_then(|md| md.modified())
            .ok()
            .filter(|&last| {
                if let Some(prev) = self.last {
                    return last > prev;
                }
                true
            }) {
            Some(time) => {
                log::debug!("FileStore changed");
                self.last.replace(time);
                true
            }
            None => false,
        }
    }
}

/// A partial Template store
///
/// This combines two `TemplateStore`s into a single store.
///
/// The `Partial` store is tried first. If it couldn't produce a valid template
/// mapping then the `Default` is attempted.
pub struct PartialStore<D, P> {
    default: D,
    partial: P,
}

impl<D, P> PartialStore<D, P> {
    /// Create a new `PartialStore` from a default `TemplateStore` and a partial `TemplateStore`
    pub fn new(default: D, partial: P) -> Self
    where
        D: TemplateStore,
        P: TemplateStore,
    {
        Self { default, partial }
    }

    /// Get a reference to the efault template store
    pub const fn default(&self) -> &D {
        &self.default
    }

    /// Get a mutable reference to the efault template store
    pub fn default_mut(&mut self) -> &mut D {
        &mut self.default
    }

    /// Get a reference to the partial template store
    pub const fn partial(&self) -> &P {
        &self.partial
    }

    /// Get a mutable reference to the partial template store
    pub fn partial_mut(&mut self) -> &mut P {
        &mut self.partial
    }

    /// Consume this wrapper, returning the default and partial stores
    pub fn into_inner(self) -> (D, P) {
        (self.default, self.partial)
    }
}

impl<D: TemplateStore, P: TemplateStore> TemplateStore for PartialStore<D, P> {
    fn parse_map(&mut self) -> Result<TemplateMap<String>, Error> {
        let left = self.partial.parse_map().unwrap_or_default();
        log::trace!("got: partial entries: {}", left.len());
        let mut right = self.default.parse_map()?;
        log::trace!("got: default entries: {}", left.len());
        right.extend(left);
        log::trace!("after merge: total: {}", right.len());
        Ok(right)
    }

    fn changed(&mut self) -> bool {
        // this will only check the partial. the default should never change (while running)
        self.partial.changed()
    }
}

impl<D, P> std::fmt::Debug for PartialStore<D, P>
where
    D: std::fmt::Debug,
    P: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PartialStore")
            .field("default", &self.default)
            .field("partial", &self.partial)
            .finish()
    }
}

/// A memory-backed store for a template
pub struct MemoryStore {
    data: String,
    changed: bool,
    loader: LoadFunction,
}

impl std::fmt::Debug for MemoryStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryStore")
            .field("data", &self.data)
            .field("changed", &self.changed)
            .finish()
    }
}

impl MemoryStore {
    /// Create a new store for the templates in `data`
    pub fn new(data: impl Into<String>, loader: LoadFunction) -> Self {
        Self {
            data: data.into(),
            changed: true,
            loader,
        }
    }

    /// Update the templates with `data` (replaces it)
    pub fn update(&mut self, data: impl Into<String>) {
        self.changed = true;
        self.data = data.into()
    }
}

impl TemplateStore for MemoryStore {
    fn parse_map(&mut self) -> Result<TemplateMap<String>, Error> {
        self.changed = false;
        (self.loader)(&self.data)
    }

    fn changed(&mut self) -> bool {
        self.changed
    }
}

/// A store that always returns an error
#[derive(Clone, Copy, Default, Debug)]
pub struct NullStore {}

impl NullStore {
    /// Create a new NullStore
    pub const fn new() -> Self {
        Self {}
    }
}

impl TemplateStore for NullStore {
    fn parse_map(&mut self) -> Result<TemplateMap<String>, Error> {
        Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "NullStore will always be empty",
        )))
    }

    fn changed(&mut self) -> bool {
        false
    }
}

impl<T> TemplateStore for Option<T>
where
    T: TemplateStore,
{
    fn parse_map(&mut self) -> Result<TemplateMap<String>, Error> {
        self.as_mut()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "None store always returns an error",
                )
            })?
            .parse_map()
    }

    fn changed(&mut self) -> bool {
        // TODO make this do something
        // self.as_mut().map(|s| s.changed()).unwrap_or(true)
        true
    }
}

impl<T> TemplateStore for Box<T>
where
    T: TemplateStore + ?Sized,
{
    fn parse_map(&mut self) -> Result<TemplateMap<String>, Error> {
        <T as TemplateStore>::parse_map(&mut *self)
    }
    fn changed(&mut self) -> bool {
        <T as TemplateStore>::changed(&mut *self)
    }
}

impl<'a, T> TemplateStore for &'a mut T
where
    T: TemplateStore,
{
    fn parse_map(&mut self) -> Result<TemplateMap<String>, Error> {
        <T as TemplateStore>::parse_map(&mut *self)
    }
    fn changed(&mut self) -> bool {
        <T as TemplateStore>::changed(&mut *self)
    }
}
