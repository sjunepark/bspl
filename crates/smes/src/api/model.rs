use crate::SmesError;
use cookie::CookieJar;
use image::DynamicImage;
use model::company::RepresentativeName;
use model::{company, table, ModelError};
use serde::{Deserialize, Serialize};

// region: Captcha
/// Represents a captcha which could be in the following three `State`s:
///
/// * `Unsubmitted`: The captcha has been received from smes,
///                  but has not been submitted to Nopecha for solving.
/// * `Submitted`: The captcha that has been submitted to Nopecha for solving.
/// * `Solved`: The captcha that has been solved.
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
pub(crate) struct Company {
    /// 고유번호 (Unique Number)
    pub(crate) vnia_sn: usize,
    /// 대표자명 (Representative Name)
    pub(crate) rprsv_nm: String,
    /// 본사주소 (Headquarters Address)
    pub(crate) hdofc_addr: String,
    /// 사업자번호 (Business Registration Number)
    pub(crate) bizrno: String,
    /// 기업명 (Company Name)
    pub(crate) cmp_nm: String,
    /// 업종코드 (Industry Code)
    pub(crate) indsty_cd: String,
    /// 업종 (Industry Name)
    pub(crate) indsty_nm: String,
}

impl TryFrom<Company> for table::Company {
    type Error = SmesError;

    fn try_from(value: Company) -> Result<Self, Self::Error> {
        Ok(table::Company {
            smes_id: company::Id::try_from(value.vnia_sn.to_string()).map_err(ModelError::from)?,
            representative_name: Into::<RepresentativeName>::into(value.rprsv_nm),
            headquarters_address: Into::<company::HeadquartersAddress>::into(value.hdofc_addr),
            business_registration_number: company::BusinessRegistrationNumber::try_from(
                value.bizrno,
            )
            .map_err(ModelError::from)?,
            company_name: Into::<company::CompanyName>::into(value.cmp_nm),
            industry_code: company::IndustryCode::try_from(value.indsty_cd)
                .map_err(ModelError::from)?,
            industry_name: Into::<company::IndustryName>::into(value.indsty_nm),
            created_date: None,
            updated_date: None,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Html {
    pub(crate) vnia_sn: String,
    pub(crate) html: String,
}

impl From<table::Html> for Html {
    fn from(value: table::Html) -> Self {
        Self {
            vnia_sn: value.smes_id.to_string(),
            html: value.html.into(),
        }
    }
}

impl TryFrom<Html> for table::Html {
    type Error = SmesError;

    fn try_from(value: Html) -> Result<Self, Self::Error> {
        Ok(table::Html {
            smes_id: company::Id::try_from(value.vnia_sn).map_err(ModelError::from)?,
            html: value.html.try_into().map_err(ModelError::from)?,
            created_date: None,
            updated_date: None,
        })
    }
}
