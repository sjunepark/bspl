use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Connection error: {0}")]
    Connection(#[from] ConnectionError),
    #[error("Serde error: {0}")]
    Deserialize(#[from] serde::de::value::Error),
    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Libsql error: {0}")]
    Libsql(#[from] libsql::Error),
    #[error("Model error: {0}")]
    Model(#[from] model::ModelError),
}

#[derive(Error, Debug)]
#[error("Connection error: {message}")]
pub struct ConnectionError {
    #[source]
    pub source: Option<Box<dyn std::error::Error>>,
    pub message: String,
}
