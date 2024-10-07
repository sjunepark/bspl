use validator::ValidationError;

pub(crate) fn length_10_or_empty(value: &str) -> Result<(), validator::ValidationError> {
    if value.is_empty() || value.len() == 10 {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_length"))
    }
}
