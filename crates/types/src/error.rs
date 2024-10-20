use crate::company::{BusinessRegistrationNumberError, HtmlContentError, IndustryCodeError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TypeError {
    #[error("BusinessRegistrationNumber error: {0}")]
    BusinessRegistrationNumber(#[from] BusinessRegistrationNumberError),
    #[error("Html error: {0}")]
    HtmlContent(#[from] HtmlContentError),
    #[error("IndustryCode error: {0}")]
    IndustryCode(#[from] IndustryCodeError),

    #[error("Init error: {0}")]
    Init(#[from] InitError),

    // Other errors
    #[error("Conversion error: {0}")]
    FromStr(#[from] FromStrError),
}

#[derive(Error, Debug)]
#[error("FromStr error: {message}")]
pub struct FromStrError {
    pub source: Option<Box<dyn std::error::Error>>,
    pub message: String,
}

#[derive(Error, Debug)]
#[error("{message}, received value: {value}")]
pub struct InitError {
    pub value: String,
    pub message: String,
}
