use std::borrow::Borrow;
use std::fmt::Display;
use std::hash::Hash;

use super::{Error, Mapping, TemplateMap, TemplateStore};

/// A collection of templates backed by a `TemplateStore`
#[derive(serde::Deserialize)]
pub struct Templates<S> {
    #[serde(skip)]
    store: S,
    templates: TemplateMap<String>,
}

impl<S> std::fmt::Debug for Templates<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Templates")
            .field("map", &self.templates)
            .finish()
    }
}

impl<S> Templates<S>
where
    S: TemplateStore,
{
    /// Create and initializations a collection with a store
    ///
    /// # Errors
    /// - An I/O Error if the data was to be loaded from a non-existant file
    /// - A deserialization error from the template source
    pub fn new(store: S) -> Result<Self, Error> {
        let mut this = Self {
            store,
            templates: TemplateMap::default(),
        };
        this.refresh().map(|_| this)
    }

    /// Tries to get the key (`namespace`) from the collection
    ///
    /// The returned value will let you get the value (`variant`).
    pub fn get<K: ?Sized>(&mut self, parent: &K) -> Option<&Mapping<String>>
    where
        K: Hash + Eq + Display,
        String: Borrow<K>,
    {
        self.templates.get(parent)
    }

    /// Refreshes the collection from the backing store
    ///
    /// # Errors
    /// - An I/O Error if the data was to be loaded from a non-existant file
    /// - A deserialization error from the template source
    pub fn refresh(&mut self) -> Result<(), Error> {
        if self.store.changed() {
            self.templates = self.store.parse_map()?;
            log::debug!("refreshed templates");
        }
        Ok(())
    }

    /// Get a reference to the inner store
    pub fn store(&self) -> &S {
        &self.store
    }

    /// Get a mutable reference to the inner store
    pub fn store_mut(&mut self) -> &mut S {
        &mut self.store
    }

    /// Consume this returning the inner store
    pub fn into_inner(self) -> S {
        self.store
    }
}
