use std::borrow::Borrow;
use std::fmt::Display;
use std::hash::Hash;

use super::{Error, Mapping, TemplateStore};

type TemplateMap<T> = std::collections::HashMap<T, Mapping<T>>;

/// A collection of templates backed by a `TemplateStore`
#[derive(Debug, serde::Deserialize)]
pub struct Templates<S> {
    #[serde(skip)]
    store: S,
    templates: TemplateMap<String>,
}

impl<S> Templates<S>
where
    S: TemplateStore,
{
    /// Create an empty collection with the store
    pub fn new(store: S) -> Result<Self, Error> {
        let mut this = Self {
            store,
            templates: TemplateMap::default(),
        };
        this.refresh().map(|_| this)
    }

    /// Tries to get the key from the collection
    pub fn get<K: ?Sized>(&mut self, parent: &K) -> Option<&Mapping<String>>
    where
        K: Hash + Eq + Display,
        String: Borrow<K>,
    {
        self.templates.get(parent)
    }

    /// Refreshes the collection from the backing store
    pub fn refresh(&mut self) -> Result<(), Error> {
        if self.store.changed() {
            self.templates = parse_template_map(self.store.data()?)?;
            log::debug!("refreshed templates");
        }
        Ok(())
    }
}

fn parse_template_map(input: String) -> Result<TemplateMap<String>, Error> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "serde_json")] {
            serde_json::from_str(&input).map_err(|err| Error::Deserialize(err.into()))
        }
        else if #[cfg(feature = "toml")] {
            toml::from_str(&input).map_err(|err| Error::Deserialize(err.into()))
        }
        else {
            unreachable!()
        }
    }
}
