mod header;
mod types;

use crate::error::{BuildError, ByteDecodeError, DeserializationError, UnsuccessfulResponseError};
use crate::ApiError;
use reqwest::header::{
    HeaderMap, HeaderValue, ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, CONNECTION, CONTENT_TYPE,
    HOST, ORIGIN, REFERER, USER_AGENT,
};
use std::fmt::Debug;
pub use types::{Company, ListPayload, ListPayloadBuilder, ListResponse};

#[allow(dead_code)]
#[derive(Debug)]
pub struct ListApi {
    client: reqwest::Client,
    /// The domain, including the protocol of the api
    pub domain: String,
}

impl ListApi {
    /// Returns the default instance of the ListApi
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for ListApi {
    fn default() -> Self {
        Self {
            client: reqwest::Client::builder()
                .default_headers(header::Fake::default().header().to_owned())
                .build()
                .expect("Failed to build reqwest client"),
            domain: "https://www.smes.go.kr".to_string(),
        }
    }
}

impl ListApi {
    /// Returns an error in the following cases:
    /// - Request returned a status code other than 200
    /// - Request returned a status code of 200,
    ///   but the api response body contained an invalid result value
    #[tracing::instrument(skip(self))]
    pub async fn make_request(&self, payload: &ListPayload) -> Result<ListResponse, ApiError> {
        // Send request
        let request_response = self
            .client
            // www.smes.go.kr/venturein/pbntc/searchVntrCmpAction
            .post(format!(
                "{}{}",
                &self.domain, "/venturein/pbntc/searchVntrCmpAction"
            ))
            .headers(self.fake_header())
            .json(payload)
            .send()
            .await
            .inspect_err(|e| {
                tracing::error!(?e, "Failed to send request");
            })?;
        tracing::trace!(?request_response, ?self.domain, "Received response");

        // Extract for future use.
        // This is because reqwest consumes the response when parsing the body.
        let status = request_response.status();
        let headers = request_response.headers().clone();

        // Check status code
        if !request_response.status().is_success() {
            return Err(ApiError::UnsuccessfulResponse(UnsuccessfulResponseError {
                message: "Request returned an unsuccessful status code",
                status,
                headers,
                body: None,
                source: None,
            }));
        };

        // Parse the response body
        let bytes = request_response.bytes().await.map_err(ApiError::Reqwest)?;
        let text = std::str::from_utf8(&bytes).map_err(|e| {
            ApiError::Conversion(ByteDecodeError {
                message: "Failed to convert bytes to string",
                source: Some(Box::new(e)),
            })
        })?;

        // Deserialize the request response
        let response: ListResponse = serde_json::from_slice(&bytes).map_err(|e| {
            ApiError::Deserialization(DeserializationError {
                message: "Failed to deserialize response",
                serialized: text.to_string(),
                source: Some(Box::new(e)),
            })
        })?;

        // Check if the response returned a successful result
        if !response.is_success() {
            return Err(ApiError::UnsuccessfulResponse(UnsuccessfulResponseError {
                message: "Response returned an unsuccessful result",
                status,
                headers,
                body: Some(response),
                source: None,
            }));
        }
        Ok(response)
    }

    pub async fn get_total_count(&self) -> Result<usize, ApiError> {
        let payload = ListPayloadBuilder::default().build().map_err(|e| {
            ApiError::Build(BuildError {
                message: "Failed to build payload",
                source: Some(Box::new(e)),
            })
        })?;
        let total_count = self
            .make_request(&payload)
            .await?
            .total_count
            .ok_or(ApiError::MissingExpectedField("total_count".to_string()))?;
        Ok(total_count)
    }

    fn fake_header(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/json, text/javascript, */*; q=0.01"),
        );
        headers.insert(
            ACCEPT_ENCODING,
            HeaderValue::from_static("gzip, deflate, br, zstd"),
        );
        headers.insert(
            ACCEPT_LANGUAGE,
            HeaderValue::from_static("en-US,en;q=0.9,ko-KR;q=0.8,ko;q=0.7,id;q=0.6"),
        );
        headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json; charset=UTF-8"),
        );
        headers.insert(HOST, HeaderValue::from_static("www.smes.go.kr"));
        headers.insert(ORIGIN, HeaderValue::from_static("https://www.smes.go.kr"));
        headers.insert(
            REFERER,
            HeaderValue::from_static("https://www.smes.go.kr/venturein/pbntc/searchVntrCmp"),
        );
        headers.insert("Sec-Fetch-Dest", HeaderValue::from_static("empty"));
        headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("cors"));
        headers.insert("Sec-Fetch-Site", HeaderValue::from_static("same-origin"));
        headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36"));
        headers.insert(
            "X-Requested-With",
            HeaderValue::from_static("XMLHttpRequest"),
        );
        headers.insert("dnt", HeaderValue::from_static("1"));
        headers.insert(
            "sec-ch-ua",
            HeaderValue::from_static(
                "\"Google Chrome\";v=\"129\", \"Not=A?Brand\";v=\"8\", \"Chromium\";v=\"129\"",
            ),
        );
        headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
        headers.insert("sec-ch-ua-platform", HeaderValue::from_static("\"macOS\""));
        headers.insert("sec-gpc", HeaderValue::from_static("1"));

        headers
    }
}

#[cfg(test)]
mod tests {
    use crate::list::api::types::ListPayloadBuilder;
    use crate::list::api::ListApi;
    use crate::ListResponse;
    use goldrust::{goldrust, Goldrust, ResponseSource};
    use tracing::Instrument;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, ResponseTemplate};

    #[tokio::test]
    async fn list_api_make_request_should_succeed() {
        // region: Arrange
        let test_id = utils::function_id!();
        tracing_setup::subscribe();
        let mut goldrust = goldrust!();

        let mock_server = wiremock::MockServer::start()
            .instrument(tracing::info_span!("test", ?test_id))
            .await;
        let mut api = ListApi::default();

        match goldrust.response_source {
            ResponseSource::Local => {
                let golden_file: String = std::fs::read_to_string(&goldrust.golden_file_path)
                    .inspect_err(|e| {
                        tracing::error!(?e, "Failed to read golden file");
                    })
                    .unwrap();

                Mock::given(method("POST"))
                    .and(path("/venturein/pbntc/searchVntrCmpAction"))
                    .respond_with({
                        tracing::debug!("Responding with a mock response");
                        ResponseTemplate::new(200).set_body_string(golden_file)
                    })
                    .expect(1)
                    .mount(&mock_server)
                    .instrument(tracing::info_span!("test", ?test_id))
                    .await;

                api.domain = mock_server.uri();
            }
            ResponseSource::External => {}
        }

        let payload = ListPayloadBuilder::default()
            .build()
            .inspect_err(|e| {
                tracing::error!(?e, "Failed to build payload");
            })
            .unwrap();
        // endregion: Arrange

        // region: Act
        let response = api
            .make_request(&payload)
            .instrument(tracing::info_span!("test", ?test_id))
            .await
            .inspect_err(|e| {
                tracing::error!(?e, payload=?&payload, "Failed to make request");
            })
            .unwrap();
        // endregion: Act

        // region: Assert
        const PAGE_SIZE: usize = 30;
        assert_eq!(payload.page_size, PAGE_SIZE);
        assert_eq!(
            response.data_list.clone().expect("No data list").len(),
            PAGE_SIZE
        );
        // endregion: Assert

        // region: Cleanup
        goldrust
            .save(response)
            .inspect_err(|e| {
                tracing::error!(?e, "Failed to save response");
            })
            .unwrap();
        // endregion: Cleanup
    }

    #[tokio::test]
    async fn list_api_total_count_should_succeed() {
        // region: Arrange
        let test_id = utils::function_id!();
        tracing_setup::subscribe();
        let allow_external_api_call: bool = std::env::var("GOLDRUST_ALLOW_EXTERNAL_API_CALL")
            .unwrap_or("false".to_string())
            .parse()
            .expect("Failed to parse GOLDRUST_ALLOW_EXTERNAL_API_CALL to bool");

        let mock_server = wiremock::MockServer::start()
            .instrument(tracing::info_span!("test", ?test_id))
            .await;
        let mut api = ListApi::new();
        const MOCK_TOTAL_COUNT: usize = 100;

        match allow_external_api_call {
            true => {}
            false => {
                let response = ListResponse {
                    total_count: Some(MOCK_TOTAL_COUNT),
                    now_page: None,
                    result: "SUCCESS".to_string(),
                    data_list: None,
                };

                Mock::given(method("POST"))
                    .and(path("/venturein/pbntc/searchVntrCmpAction"))
                    .respond_with(ResponseTemplate::new(200).set_body_json(&response))
                    .expect(1)
                    .mount(&mock_server)
                    .await;

                api.domain = mock_server.uri();
            }
        }
        // endregion: Arrange

        let total_count = api
            .get_total_count()
            .await
            .expect("Failed to get total count");

        match allow_external_api_call {
            true => {
                tracing::trace!(?total_count, "Total count received from an external api");
                assert!(total_count > 0);
            }
            false => {
                tracing::trace!(?total_count, "Total count received from a mock api");
                assert_eq!(total_count, MOCK_TOTAL_COUNT);
            }
        }
    }
}
