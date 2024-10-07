// todo: clean up tracing

use crate::api::base::ParsedResponse;
use crate::api::model::{Captcha, Solved, Submitted, Unsubmitted};
use crate::error::{ExternalApiError, NopechaError, NopechaErrorBody};
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
    #[cfg(test)]
    pub(crate) fn new(domain: &str) -> Self {
        NopechaApi::default();

        Self {
            domain: domain.to_string(),
            ..NopechaApi::default()
        }
    }

    /// * `image_data` - Image data encoded in base64
    /// Submit a captcha and get a captcha with a `nopecha_id`.
    /// The `nopecha_id` should be later submitted to get the answer.
    #[tracing::instrument(skip(self, captcha))]
    pub(crate) async fn submit_captcha(
        &self,
        captcha: Captcha<Unsubmitted>,
    ) -> Result<Captcha<Submitted>, SmesError> {
        let image = image_to_base64(captcha.image())?;

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

        let text = std::str::from_utf8(&response.bytes)?;
        tracing::trace!(?text, "Response from Nopecha API");

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum ApiResponse {
            Answer(Answer),
            Error(NopechaErrorBody),
        }
        #[derive(Deserialize, Debug)]
        struct Answer {
            data: Vec<String>,
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
            ApiResponse::Error(error) => Err(NopechaError::from(error).into()),
        }
    }

    // This is not in the NopechaApi
    // because we need a new instance of the api instance
    // to make multiple retries in the async environment.
    #[tracing::instrument(skip(captcha))]
    pub(crate) async fn get_answer_with_retries(
        &self,
        captcha: &Captcha<Submitted>,
        max_retry: usize,
        delay: Duration,
    ) -> Result<Captcha<Solved>, SmesError> {
        (|| async { self.get_answer(captcha.clone()).await })
            .retry(
                ConstantBuilder::default()
                    .with_delay(delay)
                    .with_max_times(max_retry),
            )
            .when(|e| matches!(e, SmesError::Nopecha(NopechaError::IncompleteJob(_))))
            .notify(|e, duration| tracing::warn!(?e, ?duration, "Retrying get_answer"))
            .await
    }
}

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
    use crate::api::cookie::parse_cookies;
    use crate::api::header::HeaderMapExt;
    use crate::api::model::Captcha;
    use backon::{ConstantBuilder, Retryable};
    use cookie::CookieJar;
    use goldrust::{goldrust, Content, Goldrust, ResponseSource};
    use tracing::Instrument;
    use wiremock::http::HeaderMap;
    use wiremock::{Mock, Request};

    mod retry {
        use super::*;

        #[tokio::test]
        // todo: make the test more accurate(align cookies, align answer values, etc.)
        async fn get_answers_with_retries_should_success_on_proper_response() {
            tracing_setup::subscribe();
            let test_id = utils::function_id!();
            let _span = tracing::info_span!("test", ?test_id).entered();

            let mock_server = wiremock::MockServer::start().in_current_span().await;
            let api = NopechaApi::new(mock_server.uri().as_str());

            mock(
                &mock_server,
                Scenario {
                    n_times: 3,
                    body: r#"{"code":14,"message":"Incomplete job"}"#.to_string(),
                },
            )
            .await;

            mock(
                &mock_server,
                Scenario {
                    n_times: 1,
                    body: r#"{"data":["160665"]}"#.to_string(),
                },
            )
            .await;

            let answer = api
                .get_answer_with_retries(
                    // todo: give proper cookies
                    &Captcha::new(DynamicImage::new_rgb8(1, 1), CookieJar::new()).submit("test"),
                    4,
                    Duration::from_secs(1),
                )
                .in_current_span()
                .await
                .expect("Failed to get answer");

            assert_eq!(answer.answer(), "160665");
        }

        struct Scenario {
            n_times: u64,
            body: String,
        }

        async fn mock(mock_server: &wiremock::MockServer, scenario: Scenario) {
            Mock::given(wiremock::matchers::method("GET"))
                .and(wiremock::matchers::path("/"))
                .respond_with(move |req: &Request| {
                    create_response_with_request(req.to_owned(), scenario.body.as_str())
                })
                .up_to_n_times(scenario.n_times)
                .expect(scenario.n_times)
                .mount(mock_server)
                .in_current_span()
                .await;
        }

        fn create_response_with_request(req: Request, body: &str) -> wiremock::ResponseTemplate {
            let response = wiremock::ResponseTemplate::new(200).set_body_string(body);

            let cookies = parse_cookies(&req.headers).expect("Failed to parse cookies");
            let response = HeaderMap::new()
                .append_cookies("/", &cookies)
                .expect("Failed to append cookies")
                .iter()
                .fold(response, |response, (name, value)| {
                    response.append_header(name, value)
                });

            response
        }
    }
    #[tokio::test]
    async fn submit_challenge_should_get_nopecha_id() {
        tracing_setup::subscribe();

        // region: Arrange
        tracing_setup::subscribe();
        let test_id = utils::function_id!();
        let _span = tracing::info_span!("test", ?test_id).entered();
        let mut goldrust = goldrust!("json");

        let mock_server = wiremock::MockServer::start().in_current_span().await;

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
                    .in_current_span()
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
            .in_current_span()
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
            .in_current_span()
            .await
            .expect("Failed to submit captcha");

        let result = {
            || async {
                api.clone()
                    .get_answer(captcha.clone())
                    .in_current_span()
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
        .in_current_span()
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

        Captcha::new(captcha_image, CookieJar::new())
    }
}
