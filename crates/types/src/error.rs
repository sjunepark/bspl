use thiserror::Error;

#[derive(Error, Debug)]
pub enum TypeError {
    #[error("chrono format parse error: {0}")]
    ChronoParse(#[from] chrono::format::ParseError),
    #[error("Init error: {0}")]
    Init(#[from] InitError),
}

#[derive(Error, Debug)]
#[error("{message}, received value: {value}")]
pub struct InitError {
    pub value: String,
    pub message: String,
}
