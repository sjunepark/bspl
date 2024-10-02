//! This module contains tests to check the behavior of how serde:
//! - Serializes/Deserializes a struct with optional fields
//! - handles the `skip_serializing_if` attribute

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct MiniListResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
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
fn deserialize_without_optional_field() {
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
    assert_eq!(json, r#"{"total_count":10,"result":"SUCCESS"}"#);
}

#[test]
fn serialize_without_optional_field() {
    let response = MiniListResponse {
        total_count: None,
        result: "SUCCESS".to_string(),
    };
    let json = serde_json::to_string(&response).unwrap();
    assert_eq!(json, r#"{"result":"SUCCESS"}"#);
}
