use derive_more::AsRef;
use reqwest::header::{
    HeaderMap, ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, CONNECTION, HOST, PRAGMA, REFERER,
    USER_AGENT,
};

#[allow(dead_code)]
#[derive(Debug, Clone, AsRef)]
pub(crate) struct FakeHeader(HeaderMap);

#[macro_export]
macro_rules! header_map {
    ($($key:expr => $value:expr),*) => {
        {
            let mut headers = HeaderMap::new();
            $(
                headers.insert(
                    $key,
                    reqwest::header::HeaderValue::from_static($value),
                );
            )*
            headers
        }
    };
}

impl FakeHeader {
    pub fn bspl_captcha_header() -> HeaderMap {
        header_map!(
            ACCEPT => "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8",
            ACCEPT_ENCODING => "gzip, deflate, br, zstd",
            ACCEPT_LANGUAGE => "en-US,en;q=0.9,ko-KR;q=0.8,ko;q=0.7,id;q=0.6",
            CONNECTION => "keep-alive",
            HOST => "www.smes.go.kr",
            PRAGMA => "no-cache",
            REFERER => "https://www.smes.go.kr/venturein/pbntc/searchVntrCmp",
            "Sec-Fetch-Dest" => "image",
            "Sec-Fetch-Mode" => "no-cors",
            "Sec-Fetch-Site" => "same-origin",
            USER_AGENT => "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36",
            "dnt" => "1",
            "sec-ch-ua" => "\"Google Chrome\";v=\"129\", \"Not=A?Brand\";v=\"8\", \"Chromium\";v=\"129\"",
            "sec-ch-ua-mobile" => "?0",
            "sec-ch-ua-platform" => "\"macOS\"",
            "sec-gpc" => "1"
        )
    }
}
