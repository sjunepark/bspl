use utils::impl_error;

#[derive(thiserror::Error, Debug)]
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
    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),
}

#[derive(Debug)]
pub struct ConnectionError {
    pub source: Option<Box<dyn std::error::Error>>,
    pub message: &'static str,
}

impl_error!(ConnectionError);

impl std::fmt::Display for ConnectionError {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Connection error: {}", self.message)
    }
}
