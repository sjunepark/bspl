use crate::company::IdError;

#[derive(thiserror::Error, Debug)]
pub enum ModelError {
    #[error("Id error: {0}")]
    Id(#[from] IdError),
}
