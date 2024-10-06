pub(crate) fn is_digits(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_digit())
}
