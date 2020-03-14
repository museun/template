//! Template stuff
//!
#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    dead_code,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

#[doc(inline)]
pub use markings;

mod mapping;
pub use mapping::Mapping;

mod templates;
pub use templates::Templates;

mod error;
pub use error::Error;

mod store;
pub use store::{FileStore, MemoryStore, NullStore, PartialStore, TemplateStore};

mod loader;
pub use loader::*;

/// A template mapping of `T` to `Mapping<T>`
pub type TemplateMap<T> = std::collections::HashMap<T, Mapping<T>>;

cfg_if::cfg_if! {
    if #[cfg(feature = "derive")] {
        #[allow(unused_imports)]
        #[macro_use]
        extern crate template_derive;
        #[doc(inline)]
        pub use template_derive::*;
    }
}

/// Template for applying a templated string to an enum variant
pub trait Template {
    /// Namespace of the template
    fn namespace() -> &'static str;
    /// Name of the template
    fn name() -> &'static str;
    /// Name of the specific variant
    fn variant(&self) -> &'static str;
    /// Apply this template string to this variant
    fn apply(&self, input: &str) -> Option<String>;
}

/// A Template Resolver
///
/// Provides a simple way to always get the latest template string for a `namespace.name`
#[derive(Debug)]
pub struct Resolver<S>
where
    S: TemplateStore,
{
    templates: Templates<S>,
}

impl<S: TemplateStore> Resolver<S> {
    /// Create a new resolver using this `TemplateStore`
    ///
    /// # Errors
    /// - Failure to load/parse the initial templates
    pub fn new(store: S) -> Result<Self, Error> {
        Templates::new(store).map(|templates| Self { templates })
    }

    /// Tries to get the template string for `namespace.name`
    pub fn resolve(&mut self, namespace: &str, name: &str) -> Option<&String> {
        self.templates.refresh().ok()?;
        self.templates.get(namespace)?.get(name)
    }

    /// Get a reference to the inner store
    pub fn store(&self) -> &S {
        self.templates.store()
    }

    /// Get a mutable reference to the inner store
    pub fn store_mut(&mut self) -> &mut S {
        self.templates.store_mut()
    }
}

/// Simple constructor for creating a `PartialStore` from two `MemoryStore`s
pub fn partial_memory_store(
    default: impl Into<String>,
    partial: impl Into<String>,
    loader: LoadFunction,
) -> PartialStore<MemoryStore, MemoryStore> {
    let default = MemoryStore::new(default, loader);
    let partial = MemoryStore::new(partial, loader);
    PartialStore::new(default, partial)
}

/// Simple constructor for creating a `PartialStore` using `FileStore`s
pub fn partial_file_store(
    default: impl Into<std::path::PathBuf>,
    partial: impl Into<std::path::PathBuf>,
    loader: LoadFunction,
) -> Result<PartialStore<FileStore, FileStore>, Error> {
    let default = FileStore::new(default.into(), loader)?;
    let partial = FileStore::new(partial.into(), loader)?;
    Ok(PartialStore::new(default, partial))
}
