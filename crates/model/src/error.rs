use crate::company::IdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Id error: {0}")]
    Id(#[from] IdError),

    #[error("Conversion error: {0}")]
    FromStr(#[from] FromStrError),
}

#[derive(Error, Debug)]
#[error("FromStr error: {message}")]
pub struct FromStrError {
    pub source: Option<Box<dyn std::error::Error>>,
    pub message: String,
}
