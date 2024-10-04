// todo: clean up tracing

use crate::api::base::ParsedResponse;
use crate::api::model::{Captcha, Solved, Submitted, Unsubmitted};
use crate::error::{ExternalApiError, UnsuccessfulResponseError};
use crate::SmesError;
use backon::{ConstantBuilder, Retryable};
use base64::engine::general_purpose;
use base64::Engine;
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::Debug;
use std::io::Cursor;
use std::time::Duration;

/// API for solving captcha using the Nopecha API
/// ref: <https://developers.nopecha.com/recognition/textcaptcha/>
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct NopechaApi {
    client: reqwest::Client,
    api_key: String,
    domain: String,
}

impl Default for NopechaApi {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: std::env::var("NOPECHA_KEY").expect("NOPECHA_KEY is not set"),
            domain: "https://api.nopecha.com".to_string(),
        }
    }
}

impl NopechaApi {
    /// * `image_data` - Image data encoded in base64
    /// Submit a captcha and get a captcha with a `nopecha_id`.
    /// The `nopecha_id` should be later submitted to get the answer.
    #[tracing::instrument(skip(self, captcha))]
    pub(crate) async fn submit_captcha(
        &self,
        captcha: Captcha<Unsubmitted>,
    ) -> Result<Captcha<Submitted>, SmesError> {
        let image = image_to_base64(&captcha.image())?;

        let payload = json!({
            "key": self.api_key,
            "type": "textcaptcha",
            "image_data": [image],
        });

        let response = self
            .client
            .post(format!("{}/", self.domain))
            .json(&payload)
            .send()
            .await?;
        let response = ParsedResponse::with_reqwest_response(response).await?;

        let answer: ChallengeAnswer = serde_json::from_slice(&response.bytes)?;
        let nopecha_id = answer.data;

        let captcha = captcha.submit(&nopecha_id);
        tracing::trace!(?captcha, "Captcha submitted");
        Ok(captcha)
    }

    #[tracing::instrument(skip(self, captcha))]
    pub(crate) async fn get_answer(
        &self,
        captcha: Captcha<Submitted>,
    ) -> Result<Captcha<Solved>, SmesError> {
        let payload = json!({
            "key": self.api_key,
            "id": captcha.nopecha_id(),
        });

        let response = self
            .client
            .get(format!("{}/", self.domain))
            // The docs require the key & id to be sent as query params and the payload
            // ref: <https://developers.nopecha.com/recognition/textcaptcha/
            .query(&payload)
            .json(&payload)
            .send()
            .await?;
        let response = ParsedResponse::with_reqwest_response(response).await?;

        #[allow(dead_code)]
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum ApiResponse {
            Answer(Answer),
            Error(Error),
        }
        #[allow(dead_code)]
        #[derive(Deserialize, Debug)]
        struct Answer {
            data: Vec<String>,
        }
        #[allow(dead_code)]
        #[derive(Deserialize, Debug)]
        struct Error {
            error: Option<usize>,
            message: String,
        }
        let api_response: ApiResponse = serde_json::from_slice(&response.bytes)?;

        match api_response {
            ApiResponse::Answer(answer) => Ok({
                let answer = answer.data[0].clone();

                if answer.is_empty() {
                    return Err(ExternalApiError {
                        message: "Nopecha API returned an empty answer",
                        source: None,
                    }
                    .into());
                }

                let captcha = captcha.solve(answer);
                tracing::trace!(?captcha, "Captcha solved");
                captcha
            }),
            ApiResponse::Error(error) => Err(UnsuccessfulResponseError {
                message: "Nopecha API returned an error",
                status: response.status,
                headers: response.headers,
                body: format!("{:?}", error),
                source: None,
            }
            .into()),
        }
    }
}

// This is not in the NopechaApi
// because we need a new instance of the api instance
// to make multiple retries in the async environment.
#[tracing::instrument]
pub(crate) async fn get_answer_with_retries(
    captcha: &Captcha<Submitted>,
    max_retry: usize,
    delay: Duration,
) -> Result<Captcha<Solved>, SmesError> {
    (|| async {
        let api = NopechaApi::default();
        api.get_answer(captcha.clone()).await
    })
    .retry(
        ConstantBuilder::default()
            .with_delay(delay)
            .with_max_times(max_retry),
    )
    .when(|e| matches!(e, SmesError::UnsuccessfulResponse { .. }))
    .notify(|e, duration| tracing::warn!(?e, ?duration, "Retrying get_answer"))
    .await
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
struct ChallengeAnswer {
    data: String,
}

#[tracing::instrument]
fn image_to_base64(image: &DynamicImage) -> Result<String, SmesError> {
    let mut bytes = Vec::new();
    image.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)?;
    let bytes_base64 = general_purpose::STANDARD.encode(&bytes);
    Ok(bytes_base64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::model::Captcha;
    use backon::{ConstantBuilder, Retryable};
    use goldrust::{goldrust, Content, Goldrust, ResponseSource};
    use tracing::Instrument;
    use wiremock::Mock;

    #[tokio::test]
    async fn submit_challenge_should_get_nopecha_id() {
        tracing_setup::subscribe();

        // region: Arrange
        let test_id = utils::function_id!();
        tracing_setup::subscribe();
        let mut goldrust = goldrust!("json");

        let mock_server = wiremock::MockServer::start()
            .instrument(tracing::info_span!("test", ?test_id))
            .await;

        if matches!(goldrust.response_source, ResponseSource::Local) {
            std::env::set_var("NOPECHA_KEY", "test");
        }
        let mut api = NopechaApi::default();

        match goldrust.response_source {
            ResponseSource::Local => {
                let golden_file =
                    std::fs::read(&goldrust.golden_file_path).expect("Failed to read golden file");

                Mock::given(wiremock::matchers::method("POST"))
                    .and(wiremock::matchers::path("/"))
                    .respond_with(wiremock::ResponseTemplate::new(200).set_body_bytes(golden_file))
                    .expect(1)
                    .mount(&mock_server)
                    .instrument(tracing::info_span!("test", ?test_id))
                    .await;

                api.domain = mock_server.uri();
            }
            ResponseSource::External => {}
        }
        // endregion: Arrange

        // region: Act
        let captcha = get_local_captcha();

        // Received captcha
        let captcha = api
            .submit_captcha(captcha)
            .instrument(tracing::info_span!("test", ?test_id))
            .await
            .expect("Failed to submit captcha");

        let nopecha_id = captcha.nopecha_id();
        // endregion: Act

        // region: Cleanup
        let challenge_answer = ChallengeAnswer {
            data: nopecha_id.to_string(),
        };
        goldrust
            .save(Content::Json(
                serde_json::to_value(&challenge_answer).expect("Failed to serialize"),
            ))
            .unwrap()
        // endregion: Cleanup
    }

    #[tokio::test]
    async fn get_answer_should_work() {
        tracing_setup::subscribe();
        let function_id = utils::function_id!();

        let allow_external_api_call = std::env::var("GOLDRUST_ALLOW_EXTERNAL_API_CALL")
            .unwrap_or("false".to_string())
            .parse::<bool>()
            .expect("Failed to parse GOLDRUST_ALLOW_EXTERNAL_API_CALL to bool");

        if !allow_external_api_call {
            return;
        }

        let api = NopechaApi::default();

        let captcha = get_local_captcha();

        let captcha = api
            .submit_captcha(captcha)
            .instrument(tracing::info_span!("test", ?function_id))
            .await
            .expect("Failed to submit captcha");

        let result = {
            || async {
                api.clone()
                    .get_answer(captcha.clone())
                    .instrument(tracing::info_span!("test", ?function_id))
                    .await
            }
        }
        .retry(
            ConstantBuilder::default()
                .with_delay(Duration::from_secs(1))
                .with_max_times(10),
        )
        .when(|e| matches!(e, SmesError::UnsuccessfulResponse { .. }))
        .notify(|e, duration| tracing::warn!(?e, ?duration, "Retrying"))
        .instrument(tracing::info_span!("test", ?function_id))
        .await
        .expect("Failed to get answer");

        assert_eq!(result.answer(), "160665");
    }

    fn get_local_captcha() -> Captcha<Unsubmitted> {
        let captcha_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/resources/captcha/160665.png"
        );
        let captcha_image = image::open(captcha_path).expect("Failed to load image");

        Captcha::new(captcha_image, vec![])
    }
}
