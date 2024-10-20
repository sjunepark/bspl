pub(crate) fn is_digits(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_digit())
}

#[tracing::instrument(skip(value))]
pub(crate) fn is_html_with_bspl(value: &str) -> bool {
    let result = value.contains("유동자산");
    if !result {
        tracing::warn!("The html does not contain '유동자산'");
    }
    result
}
