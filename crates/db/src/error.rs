use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Connection error: {0}")]
    Connection(#[from] ConnectionError),
    #[error("Serde error: {0}")]
    Deserialize(#[from] serde::de::value::Error),
    #[error("Diesel error: {0}")]
    Diesel(#[from] diesel::result::Error),
    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Type error: {0}")]
    Type(#[from] types::TypeError),
}

#[derive(Error, Debug)]
#[error("Connection error: {message}")]
pub struct ConnectionError {
    #[source]
    pub source: Option<Box<dyn std::error::Error>>,
    pub message: String,
}
