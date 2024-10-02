use crate::ListResponse;
use utils::impl_error;

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("HTTP error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Deserialization error: {0}")]
    Deserialization(#[from] DeserializationError),
    #[error("Unsuccessful response error: {0}")]
    UnsuccessfulResponse(#[from] UnsuccessfulResponseError),
    #[error("Conversion error: {0}")]
    Conversion(#[from] ByteDecodeError),
    #[error("Missing expected field: {0}")]
    MissingExpectedField(String),
    #[error("Build error: {0}")]
    Build(#[from] BuildError),
}

#[derive(Debug)]
pub struct BuildError {
    pub source: Option<Box<dyn std::error::Error>>,
    pub message: &'static str,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Build error: {}", self.message)
    }
}

impl_error!(BuildError);

#[derive(Debug)]
pub struct DeserializationError {
    pub source: Option<Box<dyn std::error::Error>>,
    pub message: &'static str,
    pub serialized: String,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl std::fmt::Display for DeserializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Deserialization error: {}", self.message)
    }
}

impl_error!(DeserializationError);

#[derive(Debug)]
pub struct UnsuccessfulResponseError {
    pub source: Option<Box<dyn std::error::Error>>,
    pub message: &'static str,
    pub status: reqwest::StatusCode,
    pub headers: reqwest::header::HeaderMap,
    pub body: Option<ListResponse>,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl std::fmt::Display for UnsuccessfulResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Unsuccessful response error: status_code: {:?}, headers: {:?}, message: {}",
            self.status, self.headers, self.message
        )
    }
}

impl_error!(UnsuccessfulResponseError);

#[derive(Debug)]
pub struct ByteDecodeError {
    pub source: Option<Box<dyn std::error::Error>>,
    pub message: &'static str,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl std::fmt::Display for ByteDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Conversion error: {}", self.message)
    }
}

impl_error!(ByteDecodeError);
