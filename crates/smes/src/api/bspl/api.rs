use crate::api::base::Api;
use crate::api::header::HeaderMapExt;
use crate::api::model::{Captcha, Html, Unsubmitted};
use crate::SmesError;
use reqwest::header::{HeaderMap, HeaderValue, SET_COOKIE};
use reqwest::{Client, Method};
use reqwest_cookie_store::CookieStoreMutex;
use std::sync::Arc;

pub struct BsplApi {
    client: Client,
    pub domain: String,
    cookie_store: Arc<CookieStoreMutex>,
}

impl Api for BsplApi {
    fn client(&self) -> &Client {
        &self.client
    }

    fn cookie_store(&self) -> &Arc<CookieStoreMutex> {
        &self.cookie_store
    }
}

impl Default for BsplApi {
    fn default() -> Self {
        let cookie_store = Arc::new(CookieStoreMutex::default());

        Self {
            client: Client::builder()
                .cookie_provider(Arc::clone(&cookie_store))
                .build()
                .expect("Failed to build reqwest client"),
            domain: "https://www.smes.go.kr".to_string(),
            cookie_store,
        }
    }
}

impl BsplApi {
    /// Get a captcha image from the smes website.
    ///
    /// The cookie information is stored with the captcha,
    /// and it will later be used to be submitted together with the answer.
    #[tracing::instrument(skip(self))]
    pub(crate) async fn get_captcha(&mut self) -> Result<Captcha<Unsubmitted>, SmesError> {
        let domain = self.domain.to_string();

        let response = self
            .request(
                Method::GET,
                &domain,
                "/venturein/pbntc/captchaImg.do",
                HeaderMap::with_bspl_captcha(),
                None,
                None,
            )
            .await?;

        let image = image::load_from_memory(&response.bytes)?;
        let cookies = response
            .headers
            .get_all(SET_COOKIE)
            .iter()
            .inspect(|cookie| {
                tracing::debug!(?cookie, "Received cookie");
            })
            .cloned()
            .collect();

        tracing::debug!("Received captcha image");
        Ok(Captcha::new(image, cookies))
    }

    // todo: Has to be tested via integration tests because of captcha solving process
    /// Get the HTML of the bspl page.
    ///
    /// You need to submit the pre-solved captcha answer together with the cookies.
    /// The smes website knows which captcha the answer belongs to by the cookies.
    #[tracing::instrument(skip(self))]
    pub(crate) async fn get_bspl_html(
        &mut self,
        cookies: Vec<HeaderValue>,
        company_id: usize,
        captcha_answer: &str,
    ) -> Result<Html, SmesError> {
        tracing::trace!("Getting bspl html");
        let domain = self.domain.to_string();

        let mut headers = HeaderMap::with_bspl();
        cookies.into_iter().for_each(|cookie| {
            headers.insert(SET_COOKIE, cookie);
        });

        let response = self
            .request(
                Method::POST,
                &domain,
                "/venturein/pbntc/searchVntrCmpDtls",
                headers,
                Some(&[
                    ("vniaSn", company_id.to_string().as_str()),
                    ("captcha", captcha_answer),
                ]),
                None,
            )
            .await?;

        let html = String::from_utf8(response.bytes.into()).inspect_err(|e| {
            tracing::error!(?e, "Failed to decode response body");
        })?;

        Ok(html)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use goldrust::{goldrust, Content, Goldrust, ResponseSource};
    use reqwest::header::CONTENT_TYPE;
    use tracing::Instrument;
    use wiremock::Mock;

    #[tokio::test]
    async fn get_captcha_image_should_get_valid_image() {
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
                    .respond_with(
                        wiremock::ResponseTemplate::new(200)
                            .set_body_bytes(golden_file)
                            .append_header(CONTENT_TYPE, "image/png")
                            .append_header(SET_COOKIE, "SESSION_TTL=20241003172138; Max-Age=1800"),
                    )
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
        let captcha_image = api.get_captcha().await.unwrap();
        // endregion: Act

        // region Assert
        let width = captcha_image.image().width();
        let height = captcha_image.image().height();
        assert!(width > 0);
        assert!(height > 0);

        // There should be cookies to map future answers to the captcha image
        assert!(!captcha_image.cookies().is_empty());
        // endregion: Assert

        // region: Cleanup
        goldrust
            .save(Content::Image(captcha_image.image().clone()))
            .expect("Failed to save image");
        // endregion: Cleanup

        assert!(false)
    }
}
