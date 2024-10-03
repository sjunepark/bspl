use crate::error::UnsuccessfulResponseError;
use crate::SmesError;
use reqwest::header::HeaderMap;
use reqwest::StatusCode;

pub(crate) struct ParsedResponse {
    pub(crate) status: StatusCode,
    pub(crate) headers: HeaderMap,
    pub(crate) bytes: bytes::Bytes,
}

impl ParsedResponse {
    pub(crate) async fn with_reqwest_response(
        response: reqwest::Response,
    ) -> Result<Self, SmesError> {
        // Extract for future use.
        // This is because reqwest consumes the response when parsing the body.
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response.bytes().await.map_err(SmesError::Reqwest)?;

        // Convert to string for error handling
        let body = std::str::from_utf8(&bytes).unwrap_or("Failed to decode bytes body");

        // Check status code
        if !status.is_success() {
            return Err(SmesError::UnsuccessfulResponse(UnsuccessfulResponseError {
                message: "Request returned an unsuccessful status code",
                status,
                headers,
                body: body.to_string(),
                source: None,
            }));
        };

        Ok(ParsedResponse {
            status,
            headers,
            bytes,
        })
    }
}

pub(crate) trait Api: Default {
    fn client(&self) -> &reqwest::Client;

    /// Default request method
    /// * `domain` - The domain to send the request to
    /// * `path` - The path to send the request to, should start with a `/`
    /// * `payload` - The payload to send with the request.
    ///
    /// Should be serializable to JSON
    async fn request(
        &self,
        method: reqwest::Method,
        domain: &str,
        path: &str,
        headers: HeaderMap,
        payload: Option<serde_json::Value>,
    ) -> Result<ParsedResponse, SmesError> {
        // Headers are set in the client with `default_headers`
        // If additional headers are necessary,
        // headers can be modified with the `header` method on the request builder
        let mut builder = self
            .client()
            // No need to use `.version(Version::HTTP_11)` as it's the default
            .request(method, format!("{}{}", domain, path))
            .headers(headers);

        if let Some(payload) = payload {
            builder = builder.json(&payload);
        }

        let response = builder.send().await?;
        tracing::trace!(?response, ?domain, "Received response");

        ParsedResponse::with_reqwest_response(response).await
    }
}
