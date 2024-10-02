use crate::utils::{
    deserialize_optional_number_from_string, serialize_number_as_string,
    serialize_optional_number_as_string,
};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::deserialize_number_from_string;

#[derive(Builder, Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
/// Payload for the request to the list API
/// Should be built using `ListPayloadBuilder`
// The current version does not expose unnecessary fields, such as
#[builder(setter(into, strip_option, skip))]
#[builder(default)]
pub struct ListPayload {
    /// Company name
    cmp_nm: String,
    /// Representative name
    rprsv_nm: String,
    /// Business registration number
    biz_r_no: String,
    /// The page number, out of the total number of pages
    #[builder(default = "1")]
    #[builder(setter)]
    pg: usize,
    /// The number of items per page
    ///
    /// On the actual website, a max amount of 30 is allowed,
    /// but the API does not enforce this limit
    #[serde(serialize_with = "serialize_number_as_string")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    #[builder(default = "30")]
    #[builder(setter)]
    pub page_size: usize,
    /// Area code
    area_cd: String,
    /// Sigungu area code
    sigungu_area_cd: String,
    /// Industry code
    indsty_cd: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct ListResponse {
    #[serde(serialize_with = "serialize_optional_number_as_string")]
    #[serde(deserialize_with = "deserialize_optional_number_from_string")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub total_count: Option<usize>,
    #[serde(serialize_with = "serialize_optional_number_as_string")]
    #[serde(deserialize_with = "deserialize_optional_number_from_string")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub now_page: Option<usize>,
    pub result: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub data_list: Option<Vec<Company>>,
}

impl ListResponse {
    #[allow(dead_code)]
    pub fn is_success(&self) -> bool {
        self.result == "SUCCESS"
    }
}

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
    pub vnia_sn: usize,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_payload_default_page_size_should_be_as_expected() {
        tracing_setup::subscribe();

        let payload = ListPayloadBuilder::default()
            .build()
            .inspect_err(|e| tracing::error!(?e, "Failed to build payload"))
            .unwrap();
        assert_eq!(payload.page_size, 30);
    }

    #[test]
    fn deserialize_list_response() {
        tracing_setup::subscribe();

        let json = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/resources/json/list.json"
        ));
        let response = serde_json::from_str::<ListResponse>(json)
            .inspect_err(|e| tracing::error!(?e, "Failed to deserialize"))
            .unwrap();
        assert_eq!(response.now_page, Some(1));
    }

    #[test]
    fn serialize_list_response() {
        tracing_setup::subscribe();

        let json = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/resources/json/list.json"
        ));
        let response = serde_json::from_str::<ListResponse>(json)
            .inspect_err(|e| tracing::error!(?e, "Failed to deserialize"))
            .unwrap();

        let serialized = serde_json::to_string(&response)
            .inspect_err(|e| tracing::error!(?e, "Failed to serialize"))
            .unwrap();

        let json = json.replace(" ", "").replace("\n", "");
        let serialized = serialized.replace(" ", "").replace("\n", "");

        assert_eq!(serialized, json);
    }

    #[test]
    fn deserialize_fail_list_response() {
        tracing_setup::subscribe();

        let json = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/resources/json/list_fail.json"
        ));
        let response = serde_json::from_str::<ListResponse>(json)
            .inspect_err(|e| tracing::error!(?e, "Failed to deserialize"))
            .unwrap();
        assert_eq!(response.now_page, None);
    }

    #[test]
    fn serialize_fail_list_response() {
        tracing_setup::subscribe();

        let json = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/resources/json/list_fail.json"
        ));
        let response = serde_json::from_str::<ListResponse>(json)
            .inspect_err(|e| tracing::error!(?e, "Failed to deserialize"))
            .unwrap();

        let serialized = serde_json::to_string(&response)
            .inspect_err(|e| tracing::error!(?e, "Failed to serialize"))
            .unwrap();

        let json = json.replace(" ", "").replace("\n", "");
        let serialized = serialized.replace(" ", "").replace("\n", "");

        assert_eq!(serialized, json);
    }
}
