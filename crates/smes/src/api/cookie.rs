use crate::SmesError;
use reqwest::header::{HeaderMap, SET_COOKIE};

pub(crate) fn parse_cookies(headers: &HeaderMap) -> Result<cookie::CookieJar, SmesError> {
    let mut jar = cookie::CookieJar::new();

    for header in headers.get_all(SET_COOKIE) {
        let header = header.to_str()?.to_string();
        let cookie = cookie::Cookie::parse(header)?;
        jar.add(cookie);
    }

    Ok(jar)
}
