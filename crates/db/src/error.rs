use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Serde error: {0}")]
    Deserialize(#[from] serde::de::value::Error),
    #[error("Diesel error: {0}")]
    Diesel(#[from] diesel::result::Error),
    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Type error: {0}")]
    Type(#[from] types::TypeError),
}
