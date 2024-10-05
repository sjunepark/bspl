use crate::SmesError;
use cookie::Cookie;
use reqwest::header::HeaderValue;

fn parse<'c>(cookie: HeaderValue) -> Result<Cookie<'c>, SmesError> {
    let cookie = Cookie::parse(cookie.to_str()?)?;

    unimplemented!("parse cookie")
}
