use cookie::ParseError;
use reqwest::header::{InvalidHeaderValue, ToStrError};
use serde::{Deserialize, Serialize};
use std::str::Utf8Error;
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SmesError {
    #[error("Build error: {0}")]
    Build(#[from] BuildError),
    #[error("Conversion error: {0}")]
    Conversion(#[from] ConversionError),
    #[error("Deserialization error: {0}")]
    Deserialization(#[from] DeserializationError),
    #[error("External API error: {0}")]
    ExternalApi(#[from] ExternalApiError),
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),
    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(#[from] InvalidHeaderValue),
    #[error("Missing expected field: {0}")]
    MissingExpectedField(String),
    #[error("Nopecha error: {0}")]
    Nopecha(#[from] NopechaError),
    #[error("Parse error: {0}")]
    CookieParse(#[from] ParseError),
    #[error("HTTP error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Unsuccessful response error: {0}")]
    UnsuccessfulResponse(#[from] UnsuccessfulResponseError),
}

#[derive(Error, Debug)]
pub enum NopechaError {
    #[error("Nopecha error: {0}")]
    IncompleteJob(NopechaErrorBody),
    #[error("Nopecha error: {0}")]
    OutOfCredit(NopechaErrorBody),
    #[error("Nopecha error: {0}")]
    Other(NopechaErrorBody),
}

impl From<NopechaErrorBody> for NopechaError {
    fn from(body: NopechaErrorBody) -> Self {
        match body.error {
            14 => NopechaError::IncompleteJob(body),
            16 => NopechaError::OutOfCredit(body),
            _ => NopechaError::Other(body),
        }
    }
}

#[derive(Error, Debug, Serialize, Deserialize, PartialEq)]
#[error("Nopecha error body: error: {error}, message: {message}")]
pub struct NopechaErrorBody {
    error: usize,
    message: String,
}

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("Utf8 error: {0}")]
    Utf8(#[from] Utf8Error),
    #[error("FromUtf8 error: {0}")]
    FromUtf8Error(#[from] FromUtf8Error),
    #[error("ToStr error: {0}")]
    ToStrError(#[from] ToStrError),
}

impl From<Utf8Error> for SmesError {
    fn from(e: Utf8Error) -> Self {
        SmesError::Conversion(ConversionError::Utf8(e))
    }
}

impl From<FromUtf8Error> for SmesError {
    fn from(e: FromUtf8Error) -> Self {
        SmesError::Conversion(ConversionError::FromUtf8Error(e))
    }
}

impl From<ToStrError> for SmesError {
    fn from(e: ToStrError) -> Self {
        SmesError::Conversion(ConversionError::ToStrError(e))
    }
}

#[derive(Error, Debug)]
#[error("External API error: {message}")]
pub struct ExternalApiError {
    pub message: &'static str,
    #[source]
    pub source: Option<Box<dyn std::error::Error>>,
}

#[derive(Error, Debug)]
#[error("Build error: {message}")]
pub struct BuildError {
    #[source]
    pub source: Option<Box<dyn std::error::Error>>,
    pub message: &'static str,
}

#[derive(Error, Debug)]
#[error("Deserialization error: {message}, serialized: {serialized}")]
pub struct DeserializationError {
    #[source]
    pub source: Option<Box<dyn std::error::Error>>,
    pub message: &'static str,
    pub serialized: String,
}

#[derive(Error, Debug)]
#[error(
    "Unsuccessful response error: {message}, status: {status}, headers: {headers:?}, body: {body}"
)]
pub struct UnsuccessfulResponseError {
    #[source]
    pub source: Option<Box<dyn std::error::Error>>,
    pub message: &'static str,
    pub status: reqwest::StatusCode,
    pub headers: reqwest::header::HeaderMap,
    pub body: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_nopecha_error_body() {
        let body = NopechaErrorBody {
            error: 14,
            message: "Incomplete job".to_string(),
        };

        let serialized = serde_json::to_string(&body).unwrap();
        assert_eq!(serialized, r#"{"error":14,"message":"Incomplete job"}"#);
    }

    #[test]
    fn deserialize_nopecha_error_body() {
        let json = r#"{"error":14,"message":"Incomplete job"}"#;
        let body: NopechaErrorBody = serde_json::from_slice(json.as_bytes()).unwrap();
        assert_eq!(
            body,
            NopechaErrorBody {
                error: 14,
                message: "Incomplete job".to_string()
            }
        );
    }
}
