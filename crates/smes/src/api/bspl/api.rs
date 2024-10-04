use crate::api::base::Api;
use crate::api::header::HeaderMapExt;
use crate::api::model::{Captcha, Html, Solved, Submitted, Unsubmitted};
use crate::api::nopecha::NopeChaApi;
use crate::{SmesError, VniaSn};
use reqwest::header::{HeaderMap, HeaderValue, SET_COOKIE};
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
    /// The entry point of getting the bspl HTMLs.
    ///
    /// * `companies` - A channel receiver of company ids
    ///
    /// This function will perform multiple operations, communicating with channels.
    /// 1. Get captcha images to solve
    /// 2. Solve the captchas and store each answer with the corresponding captcha
    /// 3. Request for the corresponding bspl HTML with the captcha answer
    ///
    /// Process #3 is independent of #1 and #2.
    /// You can submit any cookies and answer combination to get a bspl.
    ///
    /// ## Error handling
    /// Each process is not going to do any early returns on an error.
    /// The error will be stored with the captchas, so a single error cannot stop the whole process.
    /// This allows captchas or companies with errors to be retried in the future
    pub async fn get_bspl_htmls(
        &self,
        companies: crossbeam_channel::Receiver<VniaSn>,
    ) -> Result<(), SmesError> {
        const BUFFER_SIZE: usize = 8;

        let rx = self.get_captchas(BUFFER_SIZE).await;

        Ok(())
    }

    /// Fetch many captchas to solve.
    ///
    /// * `cap` - The maximum number of captchas to buffer in the channel.
    ///
    /// This function sends captchas to a channel that the receiver can consume.
    /// It does not introduce any artificial delays between requests;
    /// therefore, it may aggressively request captchas from the server if the receiver processes them quickly.
    /// The receiver should control the rate of captcha processing to avoid overwhelming the server.
    async fn get_captchas(
        &self,
        cap: usize,
    ) -> crossbeam_channel::Receiver<Result<Captcha<Unsubmitted>, SmesError>> {
        let (tx, rx) = crossbeam_channel::bounded::<Result<Captcha<Unsubmitted>, SmesError>>(cap);

        loop {
            let tx = tx.clone();
            let captcha = self.get_captcha().await;
            if let Err(e) = tx.send(captcha) {
                tracing::trace!(?e, "Failed to send captcha. The channel has been closed.");
                break;
            }
        }
        rx
    }

    async fn solve_captchas(
        &self,
        rx: crossbeam_channel::Receiver<Result<Unsubmitted, SmesError>>,
        cap: usize,
    ) -> crossbeam_channel::Receiver<Result<Solved, SmesError>> {
        let (tx, rx) = crossbeam_channel::bounded::<Result<Solved, SmesError>>(cap);
        let api = NopeChaApi::default();

        while let Ok(captcha) = rx.recv() {
            let tx = tx.clone();
            match captcha {
                Ok(captcha) => {
                    unimplemented!()
                }
                Err(_) => {
                    if let Err(e) = tx.send(captcha) {
                        tracing::trace!(?e, "Failed to send captcha. The channel has been closed.");
                        break;
                    }
                }
            }
        }

        rx
    }

    /// Get a captcha image from the smes website.
    ///
    /// The cookie information is stored with the captcha,
    /// and it will later be used to be submitted together with the answer.
    async fn get_captcha(&self) -> Result<Captcha<Unsubmitted>, SmesError> {
        let response = self
            .request(
                Method::GET,
                &self.domain,
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
            .cloned()
            .collect();

        Ok(Captcha::new(image, cookies))
    }

    // todo: Has to be tested via integration tests because of captcha solving process
    /// Get the HTML of the bspl page.
    ///
    /// You need to submit the pre-solved captcha answer together with the cookies.
    /// The smes website knows which captcha the answer belongs to by the cookies.
    async fn get_bspl_html(
        &self,
        cookies: Vec<HeaderValue>,
        company_id: &str,
        captcha_answer: &str,
    ) -> Result<Html, SmesError> {
        let mut headers = HeaderMap::with_bspl();
        cookies.into_iter().for_each(|cookie| {
            headers.insert(SET_COOKIE, cookie);
        });

        let response = self
            .request(
                Method::POST,
                &self.domain,
                "/venturein/pbntc/searchVntrCmpDtls",
                headers,
                Some(&[("vniaSn ", company_id), ("captcha", captcha_answer)]),
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
    }
}
