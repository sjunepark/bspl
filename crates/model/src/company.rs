use crate::utils::{is_digits, is_length_10_or_empty};
use crate::{bytes, string};

string!(Id, TryFrom => {
    validate(len_char_min = 7, len_char_max = 7, predicate = is_digits),
});
string!(RepresentativeName, From);

string!(HeadquartersAddress, From);

string!(BusinessRegistrationNumber, TryFrom => {
    validate(predicate = is_length_10_or_empty),
});
string!(CompanyName, From);
string!(IndustryCode, TryFrom => {
    validate(len_char_min = 5, len_char_max = 5, predicate = is_digits),
});
string!(IndustryName, From);

bytes!(HtmlContent, From);

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

    #[test]
    fn business_registration_number_should_be_ten_digits() {
        assert!(BusinessRegistrationNumber::try_from("1234567890").is_ok());
        assert!(BusinessRegistrationNumber::try_from("123456789").is_err());
        assert!(BusinessRegistrationNumber::try_new("1234567890").is_ok());
        assert!(BusinessRegistrationNumber::try_new("123456789").is_err());
    }

    #[test]
    fn business_registration_number_should_allow_empty_string() {
        assert!(BusinessRegistrationNumber::try_from("").is_ok());
        assert!(BusinessRegistrationNumber::try_new("").is_ok());
    }
}
