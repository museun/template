use std::borrow::Borrow;
use std::collections::HashMap;
use std::{fmt::Display, hash::Hash};

/// A mapping of Keys to Values
#[derive(Debug, Default, serde::Deserialize)]
pub struct Mapping<T: Hash + Eq + Sized, V = T>(HashMap<T, V>);

impl<T: Hash + Eq> Mapping<T> {
    /// Tries to get the value for the key
    pub fn get<K: ?Sized>(&self, key: &K) -> Option<&T>
    where
        K: Hash + Eq + Display,
        T: Borrow<K>,
    {
        self.0.get(key)
    }
}
