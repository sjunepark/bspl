// todo: clean up tracing

use crate::api::base::ParsedResponse;
use crate::api::model::Captcha;
use crate::error::{ExternalApiError, UnsuccessfulResponseError};
use crate::SmesError;
use base64::engine::general_purpose;
use base64::Engine;
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::Debug;
use std::io::Cursor;

/// API for solving captcha using the Nopecha API
/// ref: <https://developers.nopecha.com/recognition/textcaptcha/>
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct NopeChaApi {
    client: reqwest::Client,
    api_key: String,
    domain: String,
}

impl Default for NopeChaApi {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: std::env::var("NOPECHA_KEY").expect("NOPECHA_KEY is not set"),
            domain: "https://api.nopecha.com".to_string(),
        }
    }
}

impl NopeChaApi {
    /// * `image_data` - Image data encoded in base64
    #[tracing::instrument(skip(self))]
    /// Submit a captcha and get a captcha with a `nopecha_id`.
    /// The `nopecha_id` should be later submitted to get the answer.
    async fn submit_challenge(&self, mut captcha: Captcha) -> Result<Captcha, SmesError> {
        let image = image_to_base64(&captcha.image)?;

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

        captcha.nopecha_id = Some(answer.data);
        Ok(captcha)
    }

    #[tracing::instrument(skip(self))]
    async fn get_answer(&self, mut captcha: Captcha) -> Result<Captcha, SmesError> {
        let payload = json!({
            "key": self.api_key,
            "id": captcha.nopecha_id,
        });

        let response = self
            .client
            .get(format!("{}/", self.domain))
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

                captcha.answer = Some(answer);
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
        let mut api = NopeChaApi::default();

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
            .submit_challenge(captcha)
            .instrument(tracing::info_span!("test", ?test_id))
            .await
            .expect("Failed to submit captcha");

        let nopecha_id = captcha.nopecha_id.to_owned().expect("No nopecha_id");
        // endregion: Act

        // region: Cleanup
        let challenge_answer = ChallengeAnswer { data: nopecha_id };
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

        let api = NopeChaApi::default();

        let captcha = get_local_captcha();

        let captcha = api
            .submit_challenge(captcha)
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
                .with_delay(std::time::Duration::from_secs(1))
                .with_max_times(10),
        )
        .when(|e| matches!(e, SmesError::UnsuccessfulResponse { .. }))
        .notify(|e, duration| tracing::warn!(?e, ?duration, "Retrying"))
        .instrument(tracing::info_span!("test", ?function_id))
        .await
        .expect("Failed to get answer");

        assert_eq!(result.answer.as_ref().expect("No answer"), "160665");
    }

    fn get_local_captcha() -> Captcha {
        let captcha_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/resources/captcha/160665.png"
        );
        let captcha_image = image::open(captcha_path).expect("Failed to load image");

        Captcha {
            image: captcha_image,
            cookies: vec![],
            nopecha_id: None,
            answer: None,
        }
    }
}
