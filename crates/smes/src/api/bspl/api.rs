use crate::api::base::{Api, NoPayload};
use crate::SmesError;
use image::DynamicImage;

use crate::api::header::HeaderMapExt;
use reqwest::header::HeaderMap;
use reqwest::{Client, Method};

pub struct BsplApi {
    client: Client,
    pub domain: String,
}

impl Api for BsplApi {
    fn client(&self) -> &Client {
        &self.client
    }
}

impl Default for BsplApi {
    fn default() -> Self {
        Self {
            client: Client::builder()
                .build()
                .expect("Failed to build reqwest client"),
            domain: "https://www.smes.go.kr".to_string(),
        }
    }
}

impl BsplApi {
    pub async fn get_captcha_image(&self) -> Result<CaptchaImage, SmesError> {
        let request_response = self
            .request::<NoPayload>(
                Method::GET,
                &self.domain,
                "/venturein/pbntc/captchaImg.do",
                HeaderMap::with_bspl_captcha(),
                None,
            )
            .await?;

        let image = image::load_from_memory(&request_response.bytes)?;

        // todo: implement id
        Ok(CaptchaImage {
            id: "hi".to_string(),
            image,
            answer: None,
        })
    }
}

#[derive(Debug)]
pub struct CaptchaImage {
    pub id: String,
    pub image: DynamicImage,
    pub answer: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use goldrust::{goldrust, Content, Goldrust, ResponseSource};
    use tracing::Instrument;
    use wiremock::Mock;

    #[tokio::test]
    async fn get_captcha_image_should_get_vaild_image() {
        // region: Arrange
        let test_id = utils::function_id!();
        tracing_setup::subscribe();
        let mut goldrust = goldrust!("png");

        let mock_server = wiremock::MockServer::start()
            .instrument(tracing::info_span!("test", ?test_id))
            .await;
        let mut api = BsplApi::default();

        match goldrust.response_source {
            ResponseSource::Local => {
                let golden_file =
                    std::fs::read(&goldrust.golden_file_path).expect("Failed to read golden file");

                Mock::given(wiremock::matchers::method("GET"))
                    .and(wiremock::matchers::path("/venturein/pbntc/captchaImg.do"))
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
        let captcha_image = api.get_captcha_image().await.unwrap();
        // endregion: Act

        // region Assert
        assert_eq!(captcha_image.id, "hi");
        tracing::trace!(?captcha_image.image, "Captcha image");
        // endregion: Assert

        // region: Cleanup
        goldrust
            .save(Content::Image(captcha_image.image))
            .expect("Failed to save image");
        // endregion: Cleanup
    }
}
