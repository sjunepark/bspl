use crate::error::{NopechaError, NopechaErrorBody, ResponseError};
use crate::{api, SmesError};
use reqwest::header::{HeaderMap, CONTENT_TYPE};
use reqwest::StatusCode;

pub(crate) struct ParsedResponse {
    pub(crate) status: StatusCode,
    pub(crate) headers: HeaderMap,
    pub(crate) bytes: bytes::Bytes,
}

impl ParsedResponse {
    pub(crate) fn cookies(&self) -> Result<cookie::CookieJar, SmesError> {
        api::cookie::parse_cookies(&self.headers)
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
            // Check if the response body can be successfully deserialized to a known NopechaError
            // This is necessary
            // because nopecha api also returns a non 2XX status code with an error body.
            // If this block is not set,
            // the error will be returned as an `UnsuccessfulResponseError` below,
            // which is not helpful, losing some context.
            if let Ok(e) = serde_json::from_slice::<NopechaErrorBody>(&bytes) {
                match NopechaError::from(e) {
                    NopechaError::Other(_) => {}
                    NopechaError::IncompleteJob(e) | NopechaError::OutOfCredit(e) => {
                        return Err(NopechaError::from(e).into());
                    }
                }
            }

            return Err(SmesError::Response(ResponseError {
                message: "Request returned an unsuccessful status code",
                status,
                headers: Box::new(headers),
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
