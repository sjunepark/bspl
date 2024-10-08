use crate::error::{ConversionError, TypeConversionError};
use crate::html::Html;
use crate::SmesError;
use cookie::CookieJar;
use derive_more::Display;
use image::DynamicImage;
use model::company;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

// region: Captcha
/// Represents a captcha which could be in the following three `State`s:
///
/// * `Unsubmitted`: The captcha has been received from smes,
///                  but has not been submitted to Nopecha for solving.
/// * `Submitted`: The captcha that has been submitted to Nopecha for solving.
/// * `Solved`: The captcha that has been solved.
// todo: consider changing to pub(crate)
#[derive(Clone)]
pub(crate) struct Captcha<State> {
    image: DynamicImage,
    cookies: CookieJar,
    nopecha_id: Option<String>,
    answer: Option<String>,
    _marker: std::marker::PhantomData<State>,
}

#[derive(Clone)]
pub(crate) struct Unsubmitted;
#[derive(Clone)]
pub(crate) struct Submitted;
#[derive(Clone)]
pub(crate) struct Solved;

impl<State> Captcha<State> {
    pub(crate) fn image(&self) -> &DynamicImage {
        &self.image
    }

    pub(crate) fn cookies(&self) -> &CookieJar {
        &self.cookies
    }
}

impl Captcha<Unsubmitted> {
    pub(crate) fn new(image: DynamicImage, cookies: CookieJar) -> Self {
        Self {
            image,
            cookies,
            nopecha_id: None,
            answer: None,
            _marker: std::marker::PhantomData,
        }
    }

    pub(crate) fn submit(self, nopecha_id: &str) -> Captcha<Submitted> {
        Captcha {
            image: self.image,
            cookies: self.cookies,
            nopecha_id: Some(nopecha_id.to_string()),
            answer: None,
            _marker: std::marker::PhantomData,
        }
    }
}

impl Captcha<Submitted> {
    pub(crate) fn nopecha_id(&self) -> &str {
        self.nopecha_id
            .as_ref()
            .expect("nopecha_id is not set for Submitted Captcha")
    }

    pub(crate) fn solve(self, answer: String) -> Captcha<Solved> {
        Captcha {
            image: self.image,
            cookies: self.cookies,
            nopecha_id: self.nopecha_id,
            answer: Some(answer),
            _marker: std::marker::PhantomData,
        }
    }
}

impl Captcha<Solved> {
    pub(crate) fn answer(&self) -> &str {
        self.answer
            .as_ref()
            .expect("answer is not set for Solved Captcha")
    }
}

impl<T> std::fmt::Debug for Captcha<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Captcha")
            .field("cookies", &self.cookies)
            .field("nopecha_id", &self.nopecha_id)
            .field("answer", &self.answer)
            .finish()
    }
}
// endregion: Captcha

// region: Company
/// Represents a company with its details.
///
/// Example:
/// ```json
/// {
///   "vnia_sn": "1071180",
///   "rprsv_nm": "김성국",
///   "hdofc_addr": "경기도 김포시",
///   "bizrno": "5632000760",
///   "cmp_nm": "루키게임즈",
///   "indsty_cd": "63999",
///   "indsty_nm": "그 외 기타 정보 서비스업"
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Company {
    /// 고유번호 (Unique Number)
    pub vnia_sn: VniaSn,
    /// 대표자명 (Representative Name)
    pub rprsv_nm: String,
    /// 본사주소 (Headquarters Address)
    pub hdofc_addr: String,
    /// 사업자번호 (Business Registration Number)
    pub bizrno: String,
    /// 기업명 (Company Name)
    pub cmp_nm: String,
    /// 업종코드 (Industry Code)
    pub indsty_cd: String,
    /// 업종 (Industry Name)
    pub indsty_nm: String,
}

impl TryFrom<Company> for model::db::Company {
    type Error = SmesError;

    fn try_from(value: Company) -> Result<Self, Self::Error> {
        Ok(model::db::Company {
            smes_id: company::Id::try_from(value.vnia_sn.to_string())
                .map_err(TypeConversionError::new)
                .map_err(ConversionError::from)?,
            representative_name: company::RepresentativeName::try_from(value.rprsv_nm)
                .map_err(TypeConversionError::new)
                .map_err(ConversionError::from)?,
            headquarters_address: company::HeadquartersAddress::try_from(value.hdofc_addr)
                .map_err(TypeConversionError::new)
                .map_err(ConversionError::from)?,
            business_registration_number: company::BusinessRegistrationNumber::try_from(
                value.bizrno,
            )
            .map_err(TypeConversionError::new)
            .map_err(ConversionError::from)?,
            company_name: company::CompanyName::try_from(value.cmp_nm)
                .map_err(TypeConversionError::new)
                .map_err(ConversionError::from)?,
            industry_code: company::IndustryCode::try_from(value.indsty_cd)
                .map_err(TypeConversionError::new)
                .map_err(ConversionError::from)?,
            industry_name: company::IndustryName::try_from(value.indsty_nm)
                .map_err(TypeConversionError::new)
                .map_err(ConversionError::from)?,
            created_date: None,
            updated_date: None,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Display, Copy, PartialEq)]
pub struct VniaSn(pub usize);

impl Deref for VniaSn {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BsPl {
    pub vnia_sn: VniaSn,
    pub html: Html,
}

#[cfg(test)]
mod tests {
    #[test]
    fn vniasn_should_serialize_as_expected() {
        let vnia_sn = super::VniaSn(1071180);
        let json = serde_json::to_string(&vnia_sn).unwrap();
        assert_eq!(json, r#"1071180"#);
    }

    #[test]
    fn vniasn_should_deserialize_as_expected() {
        let json = r#"1071180"#;
        let vnia_sn: super::VniaSn = serde_json::from_str(json).unwrap();
        assert_eq!(vnia_sn, super::VniaSn(1071180));
    }

    #[test]
    fn vniasn_should_display_as_expected() {
        let vnia_sn = super::VniaSn(1071180);
        let display = format!("{}", vnia_sn);
        assert_eq!(display, "1071180");
    }
}
