use crate::base::{digit, text};

digit!(SmesId, false, 7);
digit!(DartId, false, 8);
digit!(BusinessRegistrationNumber, true, 10, {
    /// ## 사업자번호
    ///
    /// This field is a 10-digit number.
    /// It also allows empty strings, since the website provides empty strings for some companies.
});
digit!(CorporationRegistrationNumber, false, 13, {
    /// ## 법인등록번호
    ///
    /// This is a 13-digit number.
});
digit!(IndustryCode, false, 5, {
    /// ## 업종코드
    ///
    /// This field is a 5-digit number.
});
digit!(StockCode, false, 6, {
    /// ## 종목코드
    ///
    /// This field is a 6-digit number.
});

text!(RepresentativeName, false, {
    /// ## 대표자명
});
text!(HeadquartersAddress, false, {
    /// ## 본사주소
});
text!(Name, false, {
    /// ## 기업명
});
text!(IndustryName, false, {
    /// ## 업종
});
text!(HtmlContent, false, {
    /// ## HTML content
    ///
    /// While storing HTML as bytes can be beneficial for handling various encodings,
    /// we use a string representation due to the requirements of the `scraper` crate.
    ///
    /// Note: This approach assumes UTF-8 encoding.
    /// If dealing with non-UTF-8 content,
    /// additional handling may be required during the bytes-to-string conversion.
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn company_id_should_be_seven_digits() {
        assert!(SmesId::try_from("1234567").is_ok());
        assert!(SmesId::try_from("12345678").is_err());
        assert!(SmesId::try_new("1234567").is_ok());
        assert!(SmesId::try_new("123456a").is_err())
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
