pub(crate) fn is_digits(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_digit())
}

pub(crate) fn is_length_10_or_empty(value: &str) -> bool {
    value.is_empty() || value.len() == 10
}
