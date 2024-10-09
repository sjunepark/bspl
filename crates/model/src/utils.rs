pub(crate) fn is_digits(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_digit())
}

#[tracing::instrument]
pub(crate) fn is_length_10_or_empty(value: &str) -> bool {
    let result = value.is_empty() || value.len() == 10;
    if !result {
        tracing::warn!("The value is not empty and does not have a length of 10",);
    }
    result
}

#[tracing::instrument(skip(value))]
pub(crate) fn is_html_with_bspl(value: &str) -> bool {
    let result = value.contains("유동자산");
    if !result {
        tracing::warn!("The html does not contain '유동자산'");
    }
    result
}
