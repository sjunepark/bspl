use open_dart::client::OpenDartApi;

#[tokio::main]
async fn main() {
    tracing_setup::span!("main");

    let api = OpenDartApi::default();
    let corp_codes = api
        .get_corp_codes()
        .await
        .expect("Failed to get corp codes");

    println!("Got {} corp codes", corp_codes.iter().len());
}
