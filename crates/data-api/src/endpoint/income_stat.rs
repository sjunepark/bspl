use crate::client::DataApi;
use crate::model::DataApiKey;
use crate::DataApiError;
use derive_builder::Builder;
use model::company::CorporationRegistrationNumber;
use serde::{Deserialize, Serialize};

impl DataApi {
    #[tracing::instrument(skip(self))]
    pub async fn get_income_stat(
        &self,
        params: IncomeStatParams,
    ) -> Result<reqwest::Response, DataApiError> {
        self.get(
            "/1160100/service/GetFinaStatInfoService_V2/getIncoStat_V2",
            params,
        )
        .await
    }
}

#[derive(Builder, Debug, Serialize, Deserialize)]
#[builder(setter(into, strip_option))]
#[builder(build_fn(error = "DataApiError"))]
pub struct IncomeStatParams {
    #[serde(rename = "numOfRows")]
    num_of_rows: u64,
    #[serde(rename = "pageNo")]
    page_no: u64,
    #[serde(rename = "resultType")]
    #[builder(setter(skip))]
    #[builder(default = r#""json".to_string()"#)]
    result_type: String,
    #[serde(rename = "serviceKey")]
    #[builder(setter(skip))]
    service_key: DataApiKey,
    #[serde(rename = "crno")]
    #[builder(default)]
    crno: Option<CorporationRegistrationNumber>,
    #[serde(rename = "bizYear")]
    #[builder(default)]
    biz_year: Option<String>,
}

#[cfg(test)]
mod tests {
    use crate::model::DataApiKey;

    const SERIALIZED_PARAMS: &str = r#"{"numOfRows":10,"pageNo":1,"resultType":"json","serviceKey":"test","crno":"1234567890123"}"#;

    #[test]
    fn serialize_get_income_stat_params() {
        let params = super::IncomeStatParams {
            num_of_rows: 10,
            page_no: 1,
            result_type: "json".to_string(),
            service_key: DataApiKey::new("test"),
            crno: Some(
                "1234567890123"
                    .parse()
                    .expect("Failed to parse CorporationRegistrationNumber"),
            ),
            biz_year: None,
        };

        let serialized = serde_json::to_string(&params).unwrap();
        assert_eq!(serialized, SERIALIZED_PARAMS);
    }

    #[test]
    fn deserialize_get_income_stat_params() {
        let deserialized =
            serde_json::from_str::<super::IncomeStatParams>(SERIALIZED_PARAMS).unwrap();

        assert_eq!(deserialized.num_of_rows, 10);
        assert_eq!(deserialized.page_no, 1);
        assert_eq!(deserialized.result_type, "json");
        assert_eq!(deserialized.service_key, DataApiKey::new("test"));
        assert_eq!(deserialized.crno.unwrap().to_string(), "1234567890123");
    }
}
