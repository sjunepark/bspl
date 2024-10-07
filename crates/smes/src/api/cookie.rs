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

// This implementation is necessary to create fake `CookieJar` structs for tests,
// such as `().fake::<CookieJar>().`
#[cfg(test)]
pub(crate) mod test_impl {
    use cookie::{Cookie, CookieJar};

    pub(crate) trait CookieJarExt {
        fn fake_smes_session() -> CookieJar {
            let mut jar = CookieJar::new();

            jar.add(
                Cookie::build(("SESSION_TTL", "fake_session_ttl"))
                    .path("/")
                    .build(),
            );
            jar.add(
                Cookie::build(("SMESSESSION", "fake_smessession"))
                    .path("/")
                    .build(),
            );

            jar
        }
    }

    impl CookieJarExt for CookieJar {}
}
