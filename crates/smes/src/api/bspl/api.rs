use crate::api::base::Api;
use crate::api::header::HeaderMapExt;
use crate::api::model::{Captcha, Unsubmitted};
use crate::error::InvariantError;
use crate::html::Html;
use crate::SmesError;
use cookie::CookieJar;
use minify_html::Cfg;
use reqwest::header::HeaderMap;
use reqwest::{Client, Method};
use scraper::Selector;

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
        let cookies = response.cookies()?;
        tracing::trace!(?cookies, "Cookies from original request");

        Ok(Captcha::new(image, cookies))
    }

    // todo: Has to be tested via integration tests because of captcha solving process
    /// Get the HTML of the bspl page.
    ///
    /// You need to submit the pre-solved captcha answer together with the cookies.
    /// The smes website knows which captcha the answer belongs to by the cookies.
    #[tracing::instrument(skip(self, cookies, company_id, captcha_answer))]
    pub(crate) async fn get_bspl_html(
        &mut self,
        cookies: &CookieJar,
        company_id: &str,
        captcha_answer: &str,
    ) -> Result<Html, SmesError> {
        tracing::trace!("Getting bspl html");
        let domain = self.domain.to_string();
        const PATH: &str = "/venturein/pbntc/searchVntrCmpDtls";

        let mut headers = HeaderMap::with_bspl();
        headers.append_cookies(PATH, cookies)?;

        let response = self
            .request(
                Method::POST,
                &domain,
                PATH,
                headers,
                Some(&[("vniaSn", company_id), ("captcha", captcha_answer)]),
                None,
            )
            .await?;

        let html = minify_and_trim_html(&response.bytes)?;
        Ok(html)
    }
}

/// The function performs the following operations:
/// 1. Minify the given HTML
/// 2. Trim the HTML content to the `#real_contents` element
///
/// * `html` - The full HTML page such as <https://www.smes.go.kr/venturein/pbntc/searchVntrCmpDtls?vniaSn=1071180&captcha=302398>
/// * Returns the HTML content of the `#real_contents` element in String format.
fn minify_and_trim_html(html: &[u8]) -> Result<Html, SmesError> {
    let html = minify_html::minify(html, &Cfg::spec_compliant());
    let html = scraper::Html::parse_document(std::str::from_utf8(&html)?);
    let selector = Selector::parse("#real_contents")?;
    let mut html = html.select(&selector);

    let element = html.next().ok_or(SmesError::Invariant(InvariantError {
        source: None,
        message: "Expected at least one element with id 'real_contents'".to_string(),
    }))?;

    if let Some(element) = html.next() {
        return Err(SmesError::Invariant(InvariantError {
            source: None,
            message: format!(
                "Expected only one element with id 'real_contents', but received another {:?}",
                element
            ),
        }));
    }

    Ok(element.html())
}

#[cfg(test)]
mod tests {
    use super::*;
    use goldrust::{goldrust, Content, Goldrust, ResponseSource};
    use reqwest::header::{CONTENT_TYPE, SET_COOKIE};
    use tracing::Instrument;
    use wiremock::Mock;

    #[test]
    fn minify_and_trim_html_should_work_as_expected() {
        tracing_setup::span!("test");
        let html = r#"
            <!DOCTYPE html>
            <sript> var a = 1; </script>
            <html>
            <head>
                <title>Test</title>
            </head>
            <body>
                <div id="real_contents">
                    <a href="https://www.smes.go.kr">Link</a>
                    <p>Some text</p>
                </div>
            </body>
            </html>
        "#;

        let minified = minify_and_trim_html(html.as_bytes()).expect("Failed to minify and trim");

        assert_eq!(
            minified,
            r#"<div id="real_contents"><a href="https://www.smes.go.kr">Link</a><p>Some text</p></div>"#
        );
    }

    #[test]
    fn minify_and_trim_html_should_error_when_no_id_real_contents() {
        tracing_setup::span!("test");
        let html = r#"
            <!DOCTYPE html>
            <sript> var a = 1; </script>
            <sript>
            <html>
            <head>
                <title>Test</title>
            </head>
            <body>
                <div id="real_content">
                    <a href="https://www.smes.go.kr">Link</a>
                    <p>Some text</p>
                </div>
            </body>
            </html>
        "#;

        let minified = minify_and_trim_html(html.as_bytes());

        assert!(minified
            .inspect_err(|e| tracing::info!(
                ?e,
                "Expected error because of missing id 'real_contents'"
            ))
            .is_err(),);
    }

    #[test]
    fn minify_and_trim_html_should_fix_minor_invalid_html() {
        tracing_setup::span!("test");
        let html = r#"
            <!DOCTYPE html>
            <sript> var a = 1; </script>
            <html>
            <head>
                <title>Test</title>
            </head>
            <body>
                <div id="real_contents">
                    <a href="https://www.smes.go.kr">Link</a>
                    <p>Some text
                </div>
            </body>
            </html>
        "#;

        let minified = minify_and_trim_html(html.as_bytes()).expect("Failed to minify and trim");

        assert_eq!(
            minified,
            r#"<div id="real_contents"><a href="https://www.smes.go.kr">Link</a><p>Some text</p></div>"#
        );
    }

    #[tokio::test]
    async fn get_captcha_image_should_get_valid_image() {
        // region: Arrange
        tracing_setup::span!("#test");
        let mut goldrust = goldrust!("png");

        let mock_server = wiremock::MockServer::start().in_current_span().await;
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
                            .append_header(SET_COOKIE, "SESSION_TTL=20241003172138; Max-Age=1800; Expires=Thu, 03-Oct-2024 08:21:38 GMT; Path=/; Secure")
                            .append_header(SET_COOKIE, "SMESSESSION=52631aca-0a20-4edc-bcce-3984f073a630; Path=/venturein/; HttpOnly"),
                    )
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
        let captcha_image = api.get_captcha().await.unwrap();
        // endregion: Act

        // region Assert
        let width = captcha_image.image().width();
        let height = captcha_image.image().height();
        assert!(width > 0);
        assert!(height > 0);

        // There should be cookies to map future answers to the captcha image
        assert!(captcha_image.cookies().get("SMESSESSION").is_some());
        assert!(captcha_image.cookies().get("SESSION_TTL").is_some());
        // endregion: Assert

        // region: Cleanup
        goldrust
            .save(Content::Image(captcha_image.image().clone()))
            .expect("Failed to save image");
        // endregion: Cleanup
    }
}
