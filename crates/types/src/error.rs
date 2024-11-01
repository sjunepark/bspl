use thiserror::Error;

#[derive(Error, Debug)]
pub enum TypeError {
    #[error("chrono format parse error: {0}")]
    ChronoParse(#[from] chrono::format::ParseError),
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
}

#[derive(
    std::fmt::Debug,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    // derive_more
    derive_more::From,
    derive_more::Into,
    // serde
    serde::Serialize,
    serde::Deserialize,
    // thiserror
    Error,
)]
#[error("{self:?}")]
pub struct ValidationError {
    pub value: String,
    pub message: String,
}
