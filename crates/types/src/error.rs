use thiserror::Error;

#[derive(Error, Debug)]
pub enum TypeError {
    #[error("Init error: {0}")]
    Init(#[from] InitError),
}

#[derive(Error, Debug)]
#[error("{message}, received value: {value}")]
pub struct InitError {
    pub value: String,
    pub message: String,
}
