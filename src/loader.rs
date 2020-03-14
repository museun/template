#[allow(unused_imports)]
use crate::{Error, TemplateMap};

/// Load the `TemplateMap` from a specific format from this string
pub type LoadFunction = fn(&str) -> Result<TemplateMap<String>, Error>;

#[cfg(feature = "json")]
/// Attempts to deserialize a `TemplateMap` from this JSON string
///
/// # Errors
/// - A JSON deserialize error
pub fn load_json(input: &str) -> Result<TemplateMap<String>, Error> {
    serde_json::from_str(input).map_err(deser_err)
}

#[cfg(feature = "toml")]
/// Attempts to deserialize a `TemplateMap` from this TOML string
///
/// # Errors
/// - A TOML deserialize error
pub fn load_toml(input: &str) -> Result<TemplateMap<String>, Error> {
    serde_toml::de::from_str(input).map_err(deser_err)
}

#[cfg(feature = "yaml")]
/// Attempts to deserialize a `TemplateMap` from this YAML string
///
/// # Errors
/// - A YAML deserialize error
pub fn load_yaml(input: &str) -> Result<TemplateMap<String>, Error> {
    serde_yaml::from_str(input).map_err(deser_err)
}

#[allow(dead_code)]
#[cold]
fn deser_err(err: impl std::error::Error + Sync + Send + 'static) -> Error {
    Error::Deserialize(Box::new(err))
}
