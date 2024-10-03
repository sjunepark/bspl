use smes::{ListApi, ListPayloadBuilder};

#[tokio::main]
async fn main() {
    tracing_setup::subscribe();
    let api = ListApi::new();

    let payload = ListPayloadBuilder::default()
        .pg(1_usize)
        .page_size(30_usize)
        .build()
        .inspect_err(|e| tracing::error!(?e, "Failed to build payload"))
        .unwrap();

    let response = api
        .get_company_list(&payload)
        .await
        .inspect_err(|e| tracing::error!(?e, "Failed to make request"))
        .unwrap();

    tracing::info!(?response, "Received response");
}
