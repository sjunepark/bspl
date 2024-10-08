use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Connection error: {0}")]
    Connection(#[from] ConnectionError),
    #[error("Conversion error: {0}")]
    Conversion(#[from] ConversionError),
    #[error("Serde error: {0}")]
    Deserialize(#[from] serde::de::value::Error),
    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invariant error: {0}")]
    Invariant(#[from] InvariantError),
    #[error("Libsql error: {0}")]
    Libsql(#[from] libsql::Error),
    #[error("Model error: {0}")]
    Model(#[from] model::ModelError),
}

#[derive(Error, Debug)]
#[error("Conversion error: {message}")]
pub struct ConversionError {
    pub source: Option<Box<dyn std::error::Error>>,
    pub message: String,
}

impl ConversionError {
    pub fn new(e: impl std::error::Error + 'static) -> Self {
        Self {
            source: Some(Box::new(e)),
            message: "Conversion error".to_string(),
        }
    }
}

#[derive(Error, Debug)]
#[error("Invariant error: {message}")]
pub struct InvariantError {
    pub source: Option<Box<dyn std::error::Error>>,
    pub message: String,
}

#[derive(Error, Debug)]
#[error("Connection error: {message}")]
pub struct ConnectionError {
    #[source]
    pub source: Option<Box<dyn std::error::Error>>,
    pub message: String,
}
