use crate::error::InitError;
use crate::utils::{is_digits, is_html_with_bspl};
use crate::TypeError;
use derive_more::{AsRef, Display, From, Into};
use diesel_derive_newtype::DieselNewType;
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    // derive_more
    AsRef,
    Display,
    // serde
    Serialize,
    Deserialize,
    // diesel
    DieselNewType,
)]
pub struct Id(String);

impl Id {
    pub fn try_new(value: &str) -> Result<Self, TypeError> {
        if value.len() == 7 && is_digits(value) {
            Ok(Self(value.to_string()))
        } else {
            Err(InitError {
                value: value.to_string(),
                message: "Id must be a 7-digit number".to_string(),
            })?
        }
    }
}

impl TryFrom<&str> for Id {
    type Error = TypeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

/// 대표자명
#[derive(
    Debug,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    // derive_more
    AsRef,
    Display,
    From,
    Into,
    // serde
    Serialize,
    Deserialize,
    // diesel
    DieselNewType,
)]
pub struct RepresentativeName(String);

impl RepresentativeName {
    pub fn new(value: &str) -> Self {
        Self(value.to_string())
    }
}

/// 본사주소
#[derive(
    Debug,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    // derive_more
    AsRef,
    Display,
    From,
    Into,
    // serde
    Serialize,
    Deserialize,
    // diesel
    DieselNewType,
)]
pub struct HeadquartersAddress(String);

/// 사업자번호
///
/// This field is a 10-digit number.
/// It also allows empty strings, since the website provides empty strings for some companies.
///
/// ## Cleansing
/// Will automatically remove hyphens(-) from the input.
#[derive(
    Debug,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    // derive_more
    AsRef,
    Display,
    // serde
    Serialize,
    Deserialize,
    // diesel
    DieselNewType,
)]
pub struct BusinessRegistrationNumber(String);

impl BusinessRegistrationNumber {
    pub fn try_new(value: &str) -> Result<Self, TypeError> {
        let value = value.replace("-", "");
        if value.is_empty() || (value.len() == 10 && is_digits(&value)) {
            Ok(Self(value.to_string()))
        } else {
            Err(InitError {
                value: value.to_string(),
                message: "BusinessRegistrationNumber must be a 10-digit number".to_string(),
            })?
        }
    }
}

impl TryFrom<&str> for BusinessRegistrationNumber {
    type Error = TypeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

/// 법인등록번호
///
/// This is a 13-digit number.
///
/// ## Cleansing
/// Will automatically remove hyphens(-) from the input.
#[derive(
    Debug,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    // derive_more
    AsRef,
    Display,
    // serde
    Serialize,
    Deserialize,
    // diesel
    DieselNewType,
)]
pub struct CorporationRegistrationNumber(String);

impl CorporationRegistrationNumber {
    pub fn try_new(value: &str) -> Result<Self, TypeError> {
        let value = value.replace("-", "");
        if value.len() == 13 && is_digits(&value) {
            Ok(Self(value.to_string()))
        } else {
            Err(InitError {
                value: value.to_string(),
                message: "CorporationRegistrationNumber must be a 13-digit number".to_string(),
            })?
        }
    }
}

impl TryFrom<&str> for CorporationRegistrationNumber {
    type Error = TypeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

/// 기업명
#[derive(
    Debug,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    // derive_more
    AsRef,
    Display,
    From,
    Into,
    // serde
    Serialize,
    Deserialize,
    // diesel
    DieselNewType,
)]
pub struct Name(String);

impl Name {
    pub fn new(value: &str) -> Self {
        Self(value.to_string())
    }
}

/// 업종코드
///
/// This field is a 5-digit number.
#[derive(
    Debug,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    // derive_more
    AsRef,
    Display,
    // serde
    Serialize,
    Deserialize,
    // diesel
    DieselNewType,
)]
pub struct IndustryCode(String);

impl IndustryCode {
    pub fn try_new(value: &str) -> Result<Self, TypeError> {
        if value.len() == 5 && is_digits(value) {
            Ok(Self(value.to_string()))
        } else {
            Err(InitError {
                value: value.to_string(),
                message: "IndustryCode must be a 5-digit number".to_string(),
            })?
        }
    }
}

impl TryFrom<&str> for IndustryCode {
    type Error = TypeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

/// 업종
#[derive(
    Debug,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    // derive_more
    AsRef,
    Display,
    From,
    Into,
    // serde
    Serialize,
    Deserialize,
    // diesel
    DieselNewType,
)]
pub struct IndustryName(String);

impl IndustryName {
    pub fn new(value: &str) -> Self {
        Self(value.to_string())
    }
}

// While storing HTML as bytes can be beneficial for handling various encodings,
// we use a string representation due to the requirements of the `scraper` crate.
//
// Note: This approach assumes UTF-8 encoding.
// If dealing with non-UTF-8 content,
// additional handling may be required during the bytes-to-string conversion.
/// HTML content, represented as a UTF-8 encoded string.
#[derive(
    Debug,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    // derive_more
    AsRef,
    Display,
    // serde
    Serialize,
    Deserialize,
    // diesel
    DieselNewType,
)]
pub struct HtmlContent(String);

impl HtmlContent {
    pub fn try_new(value: &str) -> Result<Self, TypeError> {
        if is_html_with_bspl(value) {
            Ok(Self(value.to_string()))
        } else {
            Err(InitError {
                value: value.to_string(),
                message: "HtmlContent must contain '유동자산'".to_string(),
            })?
        }
    }
}

impl TryFrom<&str> for HtmlContent {
    type Error = TypeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

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
