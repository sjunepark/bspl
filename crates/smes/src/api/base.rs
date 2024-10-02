use crate::error::UnsuccessfulResponseError;
use crate::SmesError;
use reqwest::header::HeaderMap;
use reqwest::StatusCode;
use serde::Serialize;

pub(crate) struct ParsedResponse {
    pub(crate) status: StatusCode,
    pub(crate) headers: HeaderMap,
    pub(crate) bytes: bytes::Bytes,
}

pub(crate) trait Api: Default {
    fn headers() -> HeaderMap;
    fn client(&self) -> &reqwest::Client;

    /// Default request method
    /// * `domain` - The domain to send the request to
    /// * `path` - The path to send the request to, should start with a `/`
    /// * `payload` - The payload to send with the request.
    /// Should be serializable to JSON
    async fn request<P: Serialize>(
        &self,
        method: reqwest::Method,
        domain: &str,
        path: &str,
        payload: Option<&P>,
    ) -> Result<ParsedResponse, SmesError> {
        // Send request
        let mut builder = self
            .client()
            .request(method, format!("{}{}", domain, path))
            .headers(Self::headers());

        if let Some(payload) = payload {
            builder = builder.json(payload);
        }

        let response = builder.send().await?;
        tracing::trace!(?response, ?domain, "Received response");

        // Extract for future use.
        // This is because reqwest consumes the response when parsing the body.
        let status = response.status();
        let headers = response.headers().clone();

        // Check status code
        if !response.status().is_success() {
            return Err(SmesError::UnsuccessfulResponse(UnsuccessfulResponseError {
                message: "Request returned an unsuccessful status code",
                status,
                headers,
                body: None,
                source: None,
            }));
        };

        // Parse the response body
        let bytes = response.bytes().await.map_err(SmesError::Reqwest)?;

        Ok(ParsedResponse {
            status,
            headers,
            bytes,
        })
    }
}

#[macro_export]
macro_rules! impl_default_api {
    ($api_type:ty) => {
        impl Default for $api_type {
            fn default() -> Self {
                Self {
                    client: reqwest::Client::builder()
                        .default_headers(Self::headers())
                        .build()
                        .expect("Failed to build reqwest client"),
                    domain: "https://www.smes.go.kr".to_string(),
                }
            }
        }

        impl $api_type {
            pub fn new() -> Self {
                Self::default()
            }
        }
    };
}
