use crate::base::digits;

#[cfg(test)]
use fake::Fake;

// region: Digits

digits!(BusinessRegistrationNumber, true, 10, {
    /// ## 사업자번호
    ///
    /// This field is a 10-digit number.
    /// It also allows empty strings, since the website provides empty strings for some companies.
});
digits!(CorporationRegistrationNumber, false, 13, {
    /// ## 법인등록번호
    ///
    /// This is a 13-digit number.
});
digits!(DartId, false, 8);

digits!(IndustryCode, false, 5, {
    /// ## 업종코드
    ///
    /// This field is a 5-digit number.
});
digits!(SmesId, false, 7);

digits!(StockCode, false, 6, {
    /// ## 종목코드
    ///
    /// This field is a 6-digit number.
});

// endregion: Digits

// region: Text

/// ## 본사주소
pub type HeadquartersAddress = String;

// While storing HTML as bytes can be beneficial for handling various encodings,
// we use a string representation due to the requirements of the `scraper` crate.
//
// Note: This approach assumes UTF-8 encoding.
// If dealing with non-UTF-8 content,
// additional handling may be required during the bytes-to-string conversion.
/// ## SMES HTML content
///
/// The scraped HTML content of the company from SMES.
pub type SmesHtmlContent = String;

/// ## 업종
pub type IndustryName = String;

/// ## 기업명
pub type CompanyName = String;

/// ## 대표자명
pub type RepresentativeName = String;

// endregion: Text

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
