#[derive(Debug, thiserror::Error)]
pub enum DataApiError {
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Uninitialized field error: {0}")]
    UninitializedField(#[from] derive_builder::UninitializedFieldError),
}
