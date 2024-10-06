use reqwest::header::{
    HeaderMap, ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, CACHE_CONTROL, CONNECTION, CONTENT_TYPE,
    HOST, ORIGIN, PRAGMA, REFERER, USER_AGENT,
};

pub(crate) trait HeaderMapExt {
    fn with_base() -> HeaderMap;
    fn with_list() -> HeaderMap;
    fn with_bspl_captcha() -> HeaderMap;
    fn with_bspl() -> HeaderMap;
}

#[macro_export]
macro_rules! header_map {
    ($headers:expr, $($key:expr => $value:expr),*) => {
            $(
                $headers.insert(
                    $key,
                    reqwest::header::HeaderValue::from_static($value),
                );
            )*
    };
}

impl HeaderMapExt for HeaderMap {
    fn with_base() -> HeaderMap {
        let mut headers = HeaderMap::new();
        header_map!(headers,
            ACCEPT_ENCODING => "gzip, deflate, br, zstd",
            ACCEPT_LANGUAGE => "en-US,en;q=0.9,ko-KR;q=0.8,ko;q=0.7,id;q=0.6",
            CONNECTION => "keep-alive",
            HOST => "www.smes.go.kr",
            REFERER => "https://www.smes.go.kr/venturein/pbntc/searchVntrCmp",
            USER_AGENT => "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36",
            "dnt" => "1",
            "sec-ch-ua" => "\"Google Chrome\";v=\"129\", \"Not=A?Brand\";v=\"8\", \"Chromium\";v=\"129\"",
            "sec-ch-ua-mobile" => "?0",
            "sec-ch-ua-platform" => "\"macOS\"",
            "sec-gpc" => "1"
        );
        headers
    }

    fn with_list() -> HeaderMap {
        let mut headers = Self::with_base();
        header_map!(headers,
            ACCEPT => "application/json, text/javascript, */*; q=0.01",
            CONTENT_TYPE => "application/json; charset=UTF-8",
            ORIGIN => "https://www.smes.go.kr",
            "Sec-Fetch-Dest" => "empty",
            "Sec-Fetch-Mode" => "cors",
            "Sec-Fetch-Site" => "same-origin",
            "X-Requested-With" => "XMLHttpRequest"
        );
        headers
    }

    fn with_bspl_captcha() -> HeaderMap {
        let mut headers = Self::with_base();
        header_map!(headers,
            ACCEPT => "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8",
            PRAGMA => "no-cache",
            "Sec-Fetch-Dest" => "image",
            "Sec-Fetch-Mode" => "no-cors",
            "Sec-Fetch-Site" => "same-origin"
        );
        headers
    }

    fn with_bspl() -> HeaderMap {
        let mut headers = Self::with_base();
        header_map!(headers,
            ACCEPT => "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7",
            CACHE_CONTROL => "no-cache",
            PRAGMA => "no-cache",
            "Sec-Fetch-Dest" => "document",
            "Sec-Fetch-Mode" => "navigate",
            "Sec-Fetch-Site" => "same-origin",
            "Sec-Fetch-User" => "?1",
            "Upgrade-Insecure-Requests" => "1"
        );
        headers
    }
}
