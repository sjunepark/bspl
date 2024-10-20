pub(crate) fn is_digits(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_digit())
}

pub(crate) fn is_html_with_bspl(value: &str) -> bool {
    value.contains("유동자산")
}
