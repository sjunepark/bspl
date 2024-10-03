use serde::Deserialize;
use serde::Deserializer;
use serde::Serializer;
use std::fmt::Display;
use std::str::FromStr;

pub(crate) fn serialize_number_as_string<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: std::fmt::Display,
{
    serializer.serialize_str(&value.to_string())
}

pub(crate) fn serialize_optional_number_as_string<S, T>(
    value: &Option<T>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: std::fmt::Display,
{
    match value {
        Some(v) => serializer.serialize_str(&v.to_string()),
        None => serializer.serialize_none(),
    }
}

pub(crate) fn deserialize_optional_number_from_string<'de, T, D>(
    deserializer: D,
) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + Deserialize<'de>,
    <T as FromStr>::Err: Display,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt<T> {
        String(String),
        Number(T),
        None,
    }

    match StringOrInt::<T>::deserialize(deserializer)? {
        StringOrInt::String(s) => {
            if s.is_empty() {
                Ok(None)
            } else {
                s.parse::<T>().map(Some).map_err(serde::de::Error::custom)
            }
        }
        StringOrInt::Number(i) => Ok(Some(i)),
        StringOrInt::None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use serde_aux::field_attributes::deserialize_number_from_string;

    // region: OptionalNumberString
    #[derive(Debug, Serialize, Deserialize)]
    struct MiniListResponse {
        #[serde(serialize_with = "serialize_optional_number_as_string")]
        #[serde(deserialize_with = "deserialize_optional_number_from_string")]
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        total_count: Option<usize>,
        result: String,
    }

    #[test]
    fn deserialize_default() {
        let json = r#"{"total_count": 10, "result": "SUCCESS"}"#;
        let response: MiniListResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.total_count, Some(10));
        assert_eq!(response.result, "SUCCESS");
    }

    #[test]
    fn deserialize_result_for_empty_string_should_be_none() {
        let json = r#"{"total_count": "", "result": "SUCCESS"}"#;
        let response: MiniListResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.total_count, None);
        assert_eq!(response.result, "SUCCESS");
    }

    #[test]
    fn deserialize_result_for_not_existing_field_should_be_none() {
        let json = r#"{"result": "SUCCESS"}"#;
        let response: MiniListResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.total_count, None);
        assert_eq!(response.result, "SUCCESS");
    }

    #[test]
    fn serialize_default() {
        let response = MiniListResponse {
            total_count: Some(10),
            result: "SUCCESS".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert_eq!(json, r#"{"total_count":"10","result":"SUCCESS"}"#);
    }

    #[test]
    fn serialize_result_for_none_should_not_create_field() {
        let response = MiniListResponse {
            total_count: None,
            result: "SUCCESS".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert_eq!(json, r#"{"result":"SUCCESS"}"#);
    }
    // endregion

    // region: NumberString
    #[derive(Debug, Serialize, Deserialize)]
    struct NumberString {
        #[serde(serialize_with = "serialize_number_as_string")]
        #[serde(deserialize_with = "deserialize_number_from_string")]
        age: usize,
    }

    #[test]
    fn serialize_number_as_string_default() {
        let response = NumberString { age: 10 };
        let json = serde_json::to_string(&response).unwrap();
        assert_eq!(json, r#"{"age":"10"}"#);
    }

    #[test]
    fn deserialize_number_from_string_default() {
        let json = r#"{"age":"10"}"#;
        let response: NumberString = serde_json::from_str(json).unwrap();
        assert_eq!(response.age, 10);
    }
    // endregion
}
