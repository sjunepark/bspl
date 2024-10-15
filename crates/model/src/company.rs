use crate::string;
use crate::utils::{is_digits, is_html_with_bspl, is_length_10_or_empty};

string!(Id, [TryFrom] {
    /// 고유번호
    ///
    /// This field is a 7-digit number.
} => {
    validate(len_char_min = 7, len_char_max = 7, predicate = is_digits),
});

string!(RepresentativeName, [From] {
    /// 대표자명
});

string!(HeadquartersAddress, [From] {
    /// 본사주소
});

string!(BusinessRegistrationNumber, [TryFrom] {
    /// 사업자번호
    ///
    /// This field is a 10-digit number.
    /// It also allows empty strings, since the website provides empty strings for some companies.
} => {
    validate(predicate = is_length_10_or_empty),
});

string!(CorporationRegistrationNumber, [TryFrom] {
    /// 법인등록번호
    ///
    /// This field is a 13-digit number.
} => {
    validate(len_char_min = 13, len_char_max = 13, predicate = is_digits),
});

string!(CompanyName, [From] {
    /// 기업명
});

string!(IndustryCode, [TryFrom] {
    /// 업종코드
    ///
    /// This field is a 5-digit number.
} => {
    validate(len_char_min = 5, len_char_max = 5, predicate = is_digits),
});

string!(IndustryName, [From] {
    /// 업종
});

// While storing HTML as bytes can be beneficial for handling various encodings,
// we use a string representation due to the requirements of the `scraper` crate.
//
// Note: This approach assumes UTF-8 encoding.
// If dealing with non-UTF-8 content,
// additional handling may be required during the bytes-to-string conversion.
string!(HtmlContent, [TryFrom] {
    /// HTML content, represented as a UTF-8 encoded string.
} => {
    validate(predicate = is_html_with_bspl),
});

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
