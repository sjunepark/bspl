use anyhow::Context;
use base64::engine::general_purpose;
use base64::Engine;
use serde::Deserialize;
use serde_json::json;
use std::io::Cursor;
use std::path::Path;

#[allow(dead_code)]
#[derive(Debug)]
struct NopeChaApi {
    client: reqwest::Client,
    key: String,
    domain: String,
}

impl Default for NopeChaApi {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
            key: std::env::var("NOPECHA_KEY").expect("NOPECHA_KEY is not set"),
            domain: "https://api.nopecha.com".to_string(),
        }
    }
}

impl NopeChaApi {
    /// * `image_data` - Image data encoded in base64
    #[tracing::instrument(skip(self))]
    async fn submit_text_captcha(&self, image_data: String) -> anyhow::Result<String> {
        let payload = json!({
            "key": self.key,
            "type": "textcaptcha",
            "image_data": [image_data],
        });

        let response = self
            .client
            .post(format!("{}/", self.domain))
            .json(&payload)
            .send()
            .await
            .context("Failed to send request")?;
        if !response.status().is_success() {
            tracing::error!(response = ?response, "Received status code other than 2XX");
            return Err(anyhow::anyhow!("Received status code other than 2XX"));
        }

        let answer = response
            .text()
            .await
            .context("Failed to get response body")?;

        #[allow(dead_code)]
        #[derive(Deserialize)]
        struct Answer {
            data: String,
        }

        let answer: Answer = serde_json::from_str(&answer).context("Failed to parse response")?;

        Ok(answer.data)
    }

    #[tracing::instrument(skip(self))]
    async fn get_solution(&self, data_id: &str) -> anyhow::Result<String> {
        let payload = json!({
            "key": self.key,
            "id": data_id,
        });

        let response = self
            .client
            .get(format!("{}/", self.domain))
            .query(&payload)
            .json(&payload)
            .send()
            .await
            .context("Failed to send request")?;
        if !response.status().is_success() {
            let status_code = response.status().as_u16();
            let url = response.url().clone();
            let body = response
                .text()
                .await
                .context("Failed to get response body")?;
            tracing::error!(status_code, url = %url, body = ?body, "Received status code other than 2XX");
            return Err(anyhow::anyhow!("Received status code other than 2XX"));
        }

        let answer = response
            .text()
            .await
            .context("Failed to get response body")?;

        #[allow(dead_code)]
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum ApiResponse {
            Answer(Answer),
            Error(Error),
        }

        #[allow(dead_code)]
        #[derive(Deserialize, Debug)]
        struct Answer {
            data: Vec<String>,
        }

        #[allow(dead_code)]
        #[derive(Deserialize, Debug)]
        struct Error {
            error: Option<usize>,
            message: String,
        }

        let api_response: ApiResponse =
            serde_json::from_str(&answer).context("Failed to deserialize response")?;

        match api_response {
            ApiResponse::Answer(answer) => Ok(answer.data[0].clone()),
            ApiResponse::Error(error) => Err(anyhow::anyhow!("Received error: {:?}", error)),
        }
    }
}

#[tracing::instrument]
fn image_to_base64<P: AsRef<Path> + std::fmt::Debug>(path: P) -> anyhow::Result<String> {
    let image = image::open(path).context("Failed to open image")?;
    let mut bytes = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
        .context("Failed to write image to buffer")?;
    let bytes_base64 = general_purpose::STANDARD.encode(&bytes);
    Ok(bytes_base64)
}

#[cfg(test)]
mod tests {
    // todo: implement test
    #[tokio::test]
    async fn solve_captcha_should_work() -> anyhow::Result<()> {
        // TestContext::new();
        //
        // let file_name = "183949.png";
        // let path = PathBuf::from(format!("tests/captcha/{}", file_name));
        // let image_data = image_to_base64(&path).context("Failed to convert image to base64")?;
        // let api = NopeChaApi::default();
        // let data_id = api.submit_text_captcha(image_data).await?;
        // tracing::debug!(data_id = %data_id);
        // let solution = api
        //     .get_solution("etx88wijrtt5olvqkhflwqbum3urxmgkojrny64a7z1ko38h4cj876oyevs9p8q4")
        //     .await?;
        // tracing::debug!(solution = %solution);
        //
        // // assert_eq!(
        // //     solution,
        // //     path.file_stem()
        // //         .expect("Failed to get file stem")
        // //         .to_str()
        // //         .expect("Failed to convert OsStr to str")
        // // );
        Ok(())
    }
}
