/// Errors produced by this crate
#[derive(Debug)]
pub enum Error {
    /// An I/O error occurred
    Io(std::io::Error),
    /// Serialization error
    Serialize(Box<dyn std::error::Error>),
    /// Deserialization error
    Deserialize(Box<dyn std::error::Error>),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(io) => write!(f, "io error: {}", io),
            Self::Serialize(ser) => write!(f, "serialize error: {}", ser),
            Self::Deserialize(de) => write!(f, "deserialize error: {}", de),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Serialize(err) | Self::Deserialize(err) => Some(&**err),
        }
    }
}
