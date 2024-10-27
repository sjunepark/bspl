use crate::dart_common;
use crate::error::InitError;
use crate::utils::is_digits;
use crate::TypeError;
use derive_more::{AsRef, Display, From, Into};
use diesel_derive_newtype::DieselNewType;
use serde::{Deserialize, Serialize};

/// 공시구분+보고서명+기타정보
/// - \[기재정정\] : 본 보고서명으로 이미 제출된 보고서의 기재내용이 변경되어 제출된 것임
/// - \[첨부정정\] : 본 보고서명으로 이미 제출된 보고서의 첨부내용이 변경되어 제출된 것임
/// - \[첨부추가\] : 본 보고서명으로 이미 제출된 보고서의 첨부서류가 추가되어 제출된 것임
/// - \[변경등록\] : 본 보고서명으로 이미 제출된 보고서의 유동화계획이 변경되어 제출된 것임
/// - \[연장결정\] : 본 보고서명으로 이미 제출된 보고서의 신탁계약이 연장되어 제출된 것임
/// - \[발행조건확정\] : 본 보고서명으로 이미 제출된 보고서의 유가증권 발행조건이 확정되어 제출된 것임
/// - \[정정명령부과\] : 본 보고서에 대하여 금융감독원이 정정명령을 부과한 것임
/// - \[정정제출요구\] : 본 보고서에 대하여 금융감독원이 정정제출요구을 부과한 것임
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
pub struct ReportName(String);

impl ReportName {
    pub fn new(value: &str) -> Self {
        Self(value.to_string())
    }
}

/// 접수번호(14자리)
///
/// ※ 공시뷰어 연결에 이용예시
/// - PC용 : https://dart.fss.or.kr/dsaf001/main.do?rcpNo=접수번호
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
pub struct ReceiptNumber(String);

impl ReceiptNumber {
    pub fn try_new(value: &str) -> Result<Self, TypeError> {
        if value.len() == 14 && is_digits(value) {
            Ok(Self(value.to_string()))
        } else {
            Err(InitError {
                value: value.to_string(),
                message: "ReceiptNumber must be a 14-digit number".to_string(),
            })?
        }
    }
}

impl TryFrom<&str> for ReceiptNumber {
    type Error = TypeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

/// 공시 제출인명
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
pub struct FilerName(String);

impl FilerName {
    pub fn new(value: &str) -> Self {
        Self(value.to_string())
    }
}

///  공시 접수일자(YYYYMMDD)
pub type ReceiptDate = dart_common::DartDate;

/// ### 비고
///
/// 조합된 문자로 각각은 아래와 같은 의미가 있음
/// 유 : 본 공시사항은 한국거래소 유가증권시장본부 소관임
/// 코 : 본 공시사항은 한국거래소 코스닥시장본부 소관임
/// 채 : 본 문서는 한국거래소 채권상장법인 공시사항임
/// 넥 : 본 문서는 한국거래소 코넥스시장 소관임
/// 공 : 본 공시사항은 공정거래위원회 소관임
/// 연 : 본 보고서는 연결부분을 포함한 것임
/// 정 : 본 보고서 제출 후 정정신고가 있으니 관련 보고서를 참조하시기 바람
/// 철 : 본 보고서는 철회(간주)되었으니 관련 철회신고서(철회간주안내)를 참고하시기 바람
pub type Remark = dart_common::Remark;
