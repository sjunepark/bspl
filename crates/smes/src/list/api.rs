use crate::error::{BuildError, ByteDecodeError, DeserializationError, UnsuccessfulResponseError};
use crate::header::FakeHeader;
use crate::{ListPayload, ListPayloadBuilder, ListResponse, SmesError};
use std::fmt::Debug;

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
                .default_headers(FakeHeader::list_header())
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
    pub async fn get_company_list(&self, payload: &ListPayload) -> Result<ListResponse, SmesError> {
        // Send request
        let request_response = self
            .client
            // www.smes.go.kr/venturein/pbntc/searchVntrCmpAction
            .post(format!(
                "{}{}",
                &self.domain, "/venturein/pbntc/searchVntrCmpAction"
            ))
            .headers(FakeHeader::list_header())
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
            return Err(SmesError::UnsuccessfulResponse(UnsuccessfulResponseError {
                message: "Request returned an unsuccessful status code",
                status,
                headers,
                body: None,
                source: None,
            }));
        };

        // Parse the response body
        let bytes = request_response.bytes().await.map_err(SmesError::Reqwest)?;
        let text = std::str::from_utf8(&bytes).map_err(|e| {
            SmesError::Conversion(ByteDecodeError {
                message: "Failed to convert bytes to string",
                source: Some(Box::new(e)),
            })
        })?;

        // Deserialize the request response
        let response: ListResponse = serde_json::from_slice(&bytes).map_err(|e| {
            SmesError::Deserialization(DeserializationError {
                message: "Failed to deserialize response",
                serialized: text.to_string(),
                source: Some(Box::new(e)),
            })
        })?;

        // Check if the response returned a successful result
        if !response.is_success() {
            return Err(SmesError::UnsuccessfulResponse(UnsuccessfulResponseError {
                message: "Response returned an unsuccessful result",
                status,
                headers,
                body: Some(response),
                source: None,
            }));
        }
        Ok(response)
    }

    pub async fn get_company_list_count(&self) -> Result<usize, SmesError> {
        let payload = ListPayloadBuilder::default().build().map_err(|e| {
            SmesError::Build(BuildError {
                message: "Failed to build payload",
                source: Some(Box::new(e)),
            })
        })?;
        let total_count = self
            .get_company_list(&payload)
            .await?
            .total_count
            .ok_or(SmesError::MissingExpectedField("total_count".to_string()))?;
        Ok(total_count)
    }
}

#[cfg(test)]
mod tests {
    use crate::{ListApi, ListPayloadBuilder, ListResponse};
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
            .get_company_list(&payload)
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
            .get_company_list_count()
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
