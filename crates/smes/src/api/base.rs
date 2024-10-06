use crate::error::UnsuccessfulResponseError;
use crate::SmesError;
use reqwest::header::{HeaderMap, CONTENT_TYPE, SET_COOKIE};
use reqwest::StatusCode;

pub(crate) struct ParsedResponse {
    pub(crate) status: StatusCode,
    pub(crate) headers: HeaderMap,
    pub(crate) bytes: bytes::Bytes,
}

impl ParsedResponse {
    pub(crate) fn cookies(&self) -> Result<cookie::CookieJar, SmesError> {
        let mut jar = cookie::CookieJar::new();

        for header in self.headers.get_all(SET_COOKIE) {
            let header = header.to_str()?.to_string();
            let cookie = cookie::Cookie::parse(header)?;
            jar.add(cookie);
        }

        Ok(jar)
    }

    pub(crate) async fn with_reqwest_response(
        response: reqwest::Response,
    ) -> Result<Self, SmesError> {
        // Extract for future use.
        // This is because reqwest consumes the response when parsing the body.
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response.bytes().await.map_err(SmesError::Reqwest)?;

        //  Check if image
        let content_type = headers.get(CONTENT_TYPE).map(|v| v.to_str()).transpose()?;

        let body = match content_type {
            Some(content_type) if content_type.contains("image/") => "image data",
            _ => std::str::from_utf8(&bytes).inspect_err(|e| {
                tracing::error!(?e, "Failed to decode response body");
            })?,
        };

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
    /// * `domain` - The domain to send the request to.
    /// * `path` - The path to send the request to, should start with a `/`.
    /// * `headers` - The headers to send with the request, including the cookies.
    /// * `payload` - The payload to send with the request (Should be serializable to JSON).
    #[tracing::instrument(skip(self))]
    async fn request(
        &self,
        method: reqwest::Method,
        domain: &str,
        path: &str,
        headers: HeaderMap,
        query: Option<&[(&str, &str)]>,
        payload: Option<serde_json::Value>,
    ) -> Result<ParsedResponse, SmesError> {
        tracing::trace!(?headers, "Sending request");

        // Headers are set in the client with `default_headers`
        // If additional headers are necessary,
        // headers can be modified with the `header` method on the request builder
        let mut builder = self
            .client()
            // No need to use `.version(Version::HTTP_11)` as it's the default
            .request(method, format!("{}{}", domain, path))
            .headers(headers);

        if let Some(query) = query {
            builder = builder.query(&query);
        }

        if let Some(payload) = payload {
            builder = builder.json(&payload);
        }

        let response = builder.send().await?;
        tracing::trace!(?response, "Request sent and received response");

        ParsedResponse::with_reqwest_response(response).await
    }
}
