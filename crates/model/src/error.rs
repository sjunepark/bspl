use crate::company::{
    BusinessRegistrationNumberError, HtmlContentError, IdError, IndustryCodeError,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModelError {
    // Validation errors
    #[error("BusinessRegistrationNumber error: {0}")]
    BusinessRegistrationNumber(#[from] BusinessRegistrationNumberError),
    #[error("Html error: {0}")]
    HtmlContent(#[from] HtmlContentError),
    #[error("Id error: {0}")]
    Id(#[from] IdError),
    #[error("IndustryCode error: {0}")]
    IndustryCode(#[from] IndustryCodeError),

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
