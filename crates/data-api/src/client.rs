use crate::error::DataApiError;
use serde::Serialize;

pub struct DataApi {
    client: reqwest::Client,
    config: DataApiConfig,
}

pub struct DataApiConfig {
    domain: String,
}

impl DataApi {
    pub fn new(config: DataApiConfig) -> Self {
        Self {
            client: reqwest::Client::builder()
                .build()
                .expect("Failed to build reqwest client"),
            config,
        }
    }

    pub(crate) async fn get<P>(
        &self,
        path: &str,
        params: P,
    ) -> Result<reqwest::Response, DataApiError>
    where
        P: Serialize,
    {
        let url = format!("{}{}", self.config.domain, path);
        let request = self.client.get(&url).query(&params).build()?;
        tracing::trace!(?request, "Sending request");

        let response = self.client.execute(request).await?;
        Ok(response)
    }
}

impl Default for DataApi {
    fn default() -> Self {
        Self::new(DataApiConfig::default())
    }
}

impl Default for DataApiConfig {
    fn default() -> Self {
        Self {
            domain: "http://apis.data.go.kr".to_string(),
        }
    }
}
