use crate::html::Html;
use derive_more::Display;
use image::DynamicImage;
use reqwest::header::HeaderValue;
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
    cookies: Vec<HeaderValue>,
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

    pub(crate) fn cookies(&self) -> &Vec<HeaderValue> {
        &self.cookies
    }
}

impl Captcha<Unsubmitted> {
    pub(crate) fn new(image: DynamicImage, cookies: Vec<HeaderValue>) -> Self {
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

#[derive(Debug, Serialize, Deserialize, Clone, Display, Copy)]
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
