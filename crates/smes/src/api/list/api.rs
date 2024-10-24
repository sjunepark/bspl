use crate::api::base::Api;
use crate::api::header::HeaderMapExt;
use crate::error::{BuildError, DeserializationError, ResponseError};
use crate::{ListPayload, ListPayloadBuilder, ListResponse, SmesError};
use reqwest::header::HeaderMap;
use reqwest::{Client, Method};
use std::fmt::Debug;

#[derive(Debug)]
pub struct ListApi {
    client: Client,
    /// The domain, including the protocol of the api
    pub domain: String,
}

impl Api for ListApi {
    fn client(&self) -> &Client {
        &self.client
    }
}

impl Default for ListApi {
    fn default() -> Self {
        Self {
            client: Client::builder()
                .default_headers(HeaderMap::with_list())
                .build()
                .expect("Failed to build reqwest client"),
            domain: "https://www.smes.go.kr".to_string(),
        }
    }
}

impl ListApi {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns an error in the following cases:
    /// - Request returned a status code other than 200
    /// - Request returned a status code of 200,
    ///   but the api response body contained an invalid result value
    #[tracing::instrument(skip(self))]
    pub async fn get_company_list(
        &mut self,
        payload: &ListPayload,
    ) -> Result<ListResponse, SmesError> {
        let domain = self.domain.to_string();

        let request_response = self
            .request(
                Method::POST,
                &domain,
                "/venturein/pbntc/searchVntrCmpAction",
                HeaderMap::with_list(),
                None,
                Some(serde_json::to_value(payload)?),
            )
            .await?;

        let text = std::str::from_utf8(&request_response.bytes).inspect_err(|e| {
            tracing::error!(?e, "Failed to decode response body");
        })?;

        // Deserialize the request response
        let response: ListResponse =
            serde_json::from_slice(&request_response.bytes).map_err(|e| {
                Into::<SmesError>::into(DeserializationError {
                    message: "Failed to deserialize response",
                    serialized: text.to_string(),
                    source: Some(e.into()),
                })
            })?;

        // Check if the response returned a successful result
        if !response.is_success() {
            return Err(Into::<SmesError>::into(ResponseError {
                message: "Response returned an unsuccessful result",
                status: request_response.status,
                headers: Box::new(request_response.headers),
                body: text.to_string(),
                source: None,
            }));
        }
        Ok(response)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_company_list_count(&mut self) -> Result<usize, SmesError> {
        let payload = ListPayloadBuilder::default().build().map_err(|e| {
            SmesError::Build(BuildError {
                message: "Failed to build payload",
                source: Some(e.into()),
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
    use goldrust::{goldrust, Content, Goldrust, ResponseSource};
    use tracing::Instrument;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, ResponseTemplate};

    #[tokio::test]
    async fn list_api_make_request_should_succeed() {
        // region: Arrange
        tracing_setup::span!("test");
        let mut goldrust = goldrust!("json");

        let mock_server = wiremock::MockServer::start().in_current_span().await;
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
                    .in_current_span()
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
            .in_current_span()
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
            .save(Content::Json(
                serde_json::to_value(response).expect("Failed to convert to serde_json::Value"),
            ))
            .inspect_err(|e| {
                tracing::error!(?e, "Failed to save response");
            })
            .unwrap();
        // endregion: Cleanup
    }

    #[tokio::test]
    async fn list_api_total_count_should_succeed() {
        // region: Arrange
        tracing_setup::span!("test");
        let test_id = utils::function_id!();
        let _span = tracing::info_span!("test", ?test_id).entered();

        let allow_external_api_call: bool = std::env::var("GOLDRUST_ALLOW_EXTERNAL_API_CALL")
            .unwrap_or("false".to_string())
            .parse()
            .expect("Failed to parse GOLDRUST_ALLOW_EXTERNAL_API_CALL to bool");

        let mock_server = wiremock::MockServer::start().in_current_span().await;
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
