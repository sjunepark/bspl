#[tokio::main]
async fn main() {
    tracing_setup::subscribe();

    let api = smes::BsplApi::default();

    let captcha_image = api
        .get_captcha_image()
        .await
        .expect("Failed to get captcha image");

    let width = captcha_image.image.width();
    let height = captcha_image.image.height();

    assert!(!captcha_image.id.is_empty());
    assert!(width > 0);
    assert!(height > 0);

    tracing::info!(?width, ?height, "Received captcha image");
}
