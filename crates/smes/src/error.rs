use std::str::Utf8Error;
use utils::impl_error;

#[derive(thiserror::Error, Debug)]
pub enum SmesError {
    #[error("Build error: {0}")]
    Build(#[from] BuildError),
    #[error("Conversion error: {0}")]
    Conversion(#[from] Utf8Error),
    #[error("Deserialization error: {0}")]
    Deserialization(#[from] DeserializationError),
    #[error("External API error: {0}")]
    ExternalApi(#[from] ExternalApiError),
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),
    #[error("Missing expected field: {0}")]
    MissingExpectedField(String),
    #[error("HTTP error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Unsuccessful response error: {0}")]
    UnsuccessfulResponse(#[from] UnsuccessfulResponseError),
}

#[derive(Debug)]
pub struct ExternalApiError {
    pub source: Option<Box<dyn std::error::Error>>,
    pub message: &'static str,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl std::fmt::Display for ExternalApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "External API error: {}", self.message)
    }
}

impl_error!(ExternalApiError);

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
    pub body: String,
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
