use image::DynamicImage;
use reqwest::header::HeaderValue;

#[derive(Clone)]
pub struct Captcha {
    pub image: DynamicImage,
    pub cookies: Vec<HeaderValue>,
    pub nopecha_id: Option<String>,
    pub answer: Option<String>,
}

impl std::fmt::Debug for Captcha {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Captcha")
            .field("cookies", &self.cookies)
            .field("nopecha_id", &self.nopecha_id)
            .field("answer", &self.answer)
            .finish()
    }
}
