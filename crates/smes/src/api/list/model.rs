use crate::utils::{
    deserialize_optional_number_from_string, serialize_number_as_string,
    serialize_optional_number_as_string,
};
use crate::SmesError;
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
    pub(crate) total_count: Option<usize>,
    #[serde(serialize_with = "serialize_optional_number_as_string")]
    #[serde(deserialize_with = "deserialize_optional_number_from_string")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub(crate) now_page: Option<usize>,
    pub(crate) result: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub(crate) data_list: Option<Vec<crate::Company>>,
}

impl ListResponse {
    pub(crate) fn is_success(&self) -> bool {
        self.result == "SUCCESS"
    }

    pub fn companies(self) -> Result<Vec<db::model::smes::NewCompany>, SmesError> {
        if let Some(data_list) = self.data_list {
            data_list.into_iter().map(|c| c.try_into()).collect()
        } else {
            Ok(Vec::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_payload_default_page_size_should_be_as_expected() {
        tracing_setup::span!("test");

        let payload = ListPayloadBuilder::default()
            .build()
            .inspect_err(|e| tracing::error!(?e, "Failed to build payload"))
            .unwrap();
        assert_eq!(payload.page_size, 30);
    }

    #[test]
    fn deserialize_list_response() {
        tracing_setup::span!("test");

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
        tracing_setup::span!("test");

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
        tracing_setup::span!("test");

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
        tracing_setup::span!("test");

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
