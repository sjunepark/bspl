use crate::SmesError;
use cookie::CookieJar;
use db::model::smes::NewHtml;
use image::DynamicImage;
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

impl TryFrom<Company> for db::model::smes::NewCompany {
    type Error = SmesError;

    fn try_from(value: Company) -> Result<Self, Self::Error> {
        Ok(db::model::smes::NewCompany {
            smes_id: value.vnia_sn.to_string().as_str().try_into()?,
            representative_name: value.rprsv_nm.into(),
            headquarters_address: value.hdofc_addr.into(),
            business_registration_number: value.bizrno.as_str().try_into()?,
            company_name: value.cmp_nm.into(),
            industry_code: value.indsty_cd.as_str().try_into()?,
            industry_name: value.indsty_nm.into(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Html {
    pub(crate) vnia_sn: String,
    pub(crate) html: String,
}

impl TryFrom<Html> for NewHtml {
    type Error = SmesError;

    fn try_from(value: Html) -> Result<Self, Self::Error> {
        Ok(NewHtml {
            smes_id: value.vnia_sn.as_str().try_into()?,
            html_content: value.html,
        })
    }
}
