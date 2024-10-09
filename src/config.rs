use serde::Deserialize;

#[derive(Deserialize)]
pub struct AppConfig {
    pub update_all_html: bool,
}
