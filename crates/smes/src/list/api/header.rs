use derive_more::AsRef;
use reqwest::header::{
    HeaderMap, ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, CONNECTION, CONTENT_TYPE, HOST, ORIGIN,
    REFERER, USER_AGENT,
};

#[allow(dead_code)]
#[derive(Debug, Clone, AsRef)]
pub struct Fake(HeaderMap);

impl Fake {
    pub fn header(&self) -> &HeaderMap {
        &self.0
    }
}

macro_rules! header {
    ($($key:expr => $value:expr),*) => {
        {
            let mut headers = HeaderMap::new();
            $(
                headers.insert(
                    $key,
                    reqwest::header::HeaderValue::from_static($value),
                );
            )*
            Fake(headers)
        }
    };
}

impl Default for Fake {
    fn default() -> Self {
        header! {
            ACCEPT => "application/json, text/javascript, */*; q=0.01",
            ACCEPT_ENCODING => "gzip, deflate, br, zstd",
            ACCEPT_LANGUAGE => "en-US,en;q=0.9,ko-KR;q=0.8,ko;q=0.7,id;q=0.6",
            CONNECTION => "keep-alive",
            CONTENT_TYPE => "application/json; charset=UTF-8",
            // COOKIE => "SMESSESSION=e7743010-1a6f-403c-8d3b-2d3e59b9375f; __VCAP_ID__=c00e9753-eb00-4f2f-7481-36a2; JSESSIONID=832C2AE92673E8C0AAB9EE2FEC56A7EB; JSESSIONID=NDpNEVGmNR548GCiwtdyX1jeKBMdddpezQGd8lx3RbPf4fJuz64x18RPyk6lqeNl.VElQQS9zbWVzXzI=; SESSION_TTL=20240915213047",
            HOST => "www.smes.go.kr",
            ORIGIN => "https://www.smes.go.kr",
            REFERER => "https://www.smes.go.kr/venturein/pbntc/searchVntrCmp",
            USER_AGENT => "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36"
        }
    }
}
