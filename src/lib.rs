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

#[cfg(feature = "derive")]
#[allow(unused_imports)]
#[cfg(feature = "derive")]
extern crate template_derive;

#[cfg(feature = "derive")]
#[doc(inline)]
pub use template_derive::*;

/**
Template for applying a templated string to an enum variant

# Example using the derive feature
```rust,ignore
use template::Template;
// first, derive Template
#[derive(Template, Debug)]
// then add a namespace, this it the 'section' (or 'object') containing each template for this type
#[namespace("response")]
enum MyResponse<'a> {
    // each field is an entry under the namespace
    Hello {
        // and each named field is the variable in the template
        name: &'a str,
    },
    CountItems { count: usize },
    // fieldless variants don't have variables
    Okay,
}

let hello = MyResponse::Hello { name: "world" };
let count = MyResponse::CountItems { count: 42 };
let okay = MyResponse::Okay;

assert_eq!(MyResponse::namespace(), "response");
// snake_case of the enum
assert_eq!(MyResponse::name(), "my_response");

// snake_case of variant
assert_eq!(hello.variant(), "hello");
assert_eq!(count.variant(), "count_items");
assert_eq!(okay.variant(), "okay");

assert_eq!(hello.apply("hello ${name}!").unwrap(), "hello world!");
assert_eq!(count.apply("count is: ${count}").unwrap(), "count is: 42");
assert_eq!(okay.apply("okay response").unwrap(), "okay response");
```

## Example of how you could store the templates in textual form:
### TOML
```toml
[response]                         # namespace
hello       = "hello ${name}!"     # Hello { name: &str }
count_items = "count is: ${count}" # CountItems { count: usize }
okay        = "okay response"      # Okay
```

### JSON
```json
{
    "response": {
        "hello": "hello ${name}!",
        "count_items": "count is: ${count}",
        "okay": "okay response"
    }
}
```

### YAML
```yaml
response:
  hello: hello ${name}!
  count_items: 'count is: ${count}'
  okay: okay response
```
*/
pub trait Template {
    /// Namespace of the template
    fn namespace() -> &'static str;
    /// Name of the template (the enum's name, in _snake_case_)
    fn name() -> &'static str;
    /// Name of the specific variant (in _snake_case_)
    fn variant(&self) -> &'static str;
    /// Apply this template string to this variant
    fn apply(&self, input: &str) -> Option<String>;
}

/// A Template Resolver
///
/// Provides a simple way to always get the latest template string for a `namespace.variant`
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

    /// Tries to get the template string for `namespace.variant`
    pub fn resolve(&mut self, namespace: &str, variant: &str) -> Option<&String> {
        self.templates
            .refresh()
            .map_err(|err| {
                log::warn!(
                    "Cannot refresh templates ({}::{}): {}",
                    namespace,
                    variant,
                    err
                );
                err
            })
            .ok()?;

        self.templates.get(namespace)?.get(variant)
    }

    /// Get a reference to the inner store
    pub fn store(&self) -> &S {
        self.templates.store()
    }

    /// Get a mutable reference to the inner store
    pub fn store_mut(&mut self) -> &mut S {
        self.templates.store_mut()
    }

    /// Get the templates
    pub fn templates(&self) -> &Templates<S> {
        &self.templates
    }

    /// Get the templates
    pub fn templates_mut(&mut self) -> &mut Templates<S> {
        &mut self.templates
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
