use crate::string;
use crate::utils::is_digits;

string!(Id => {
    validate(len_char_min = 7, len_char_max = 7, predicate = is_digits),
});

string!(HeadquartersAddress);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn company_id_should_be_seven_digits() {
        assert!(Id::try_from("1234567").is_ok());
        assert!(Id::try_from("12345678").is_err());
        assert!(Id::try_new("1234567").is_ok());
        assert!(Id::try_new("123456a").is_err())
    }
}
