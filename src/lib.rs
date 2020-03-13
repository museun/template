//! Template stuff
//!
#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

cfg_if::cfg_if! {
    if #[cfg(feature = "derive")] {
        #[allow(unused_imports)]
        #[macro_use]
        extern crate template_derive;
        #[doc(inline)]
        pub use template_derive::*;
    }
}

#[doc(inline)]
pub use markings;

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

mod mapping;
pub use mapping::Mapping;

mod templates;
pub use templates::Templates;

mod error;
pub use error::Error;

mod store;
pub use store::{FileStore, MemoryStore, TemplateStore};

// TODO make this less confusing
static_assertions::assert_cfg!(
    not(all(feature = "serde_json", feature = "toml")),
    "a single `serde_json` or `toml` feature must be chosen"
);

static_assertions::assert_cfg!(
    not(all(not(feature = "serde_json"), not(feature = "toml"))),
    "the feature `serde_json` or `toml` must be chosen"
);

impl<T> TemplateStore for Box<T>
where
    T: TemplateStore,
{
    fn data(&mut self) -> std::io::Result<String> {
        <T as TemplateStore>::data(&mut *self)
    }
    fn changed(&mut self) -> bool {
        <T as TemplateStore>::changed(&mut *self)
    }
}

// TODO unit tests
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn boxed_template_store() {
        let mut store: Box<dyn TemplateStore> = Box::new(MemoryStore::new("[foo] bar = ${baz}"));
        let _ = store.data();
        let _ = store.changed();
    }
}
