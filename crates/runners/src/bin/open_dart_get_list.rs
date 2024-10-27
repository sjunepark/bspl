use open_dart::client::OpenDartApi;
use open_dart::endpoints::list;
use tracing::Instrument;

#[tokio::main]
async fn main() {
    tracing_setup::span!("main");

    let params = list::ParamsBuilder::default()
        .bgn_de(chrono::NaiveDate::from_ymd_opt(2024, 10, 1).expect("Invalid date"))
        .build()
        .expect("Failed to build ListRequestParams");
    tracing::info!(?params, "Request parameters");

    let list = OpenDartApi::new(Default::default())
        .get_list(params)
        .in_current_span()
        .await
        .unwrap();

    if let Some(body) = list.body {
        tracing::info!("{:?}", body);
    } else {
        tracing::warn!("No content");
    }
}
