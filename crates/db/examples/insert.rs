use db::{Companies, LibsqlDb};

#[tokio::main]
async fn main() {
    tracing_setup::span!("main");

    let json = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/resources/list.json"
    ))
    .inspect_err(|e| tracing::error!(?e, "Failed to read file"))
    .unwrap();

    let list = serde_json::from_str::<smes::ListResponse>(&json)
        .inspect_err(|e| tracing::error!(?e, "Failed to deserialize"))
        .unwrap();

    let db = LibsqlDb::new_local(":memory:").await.unwrap();

    let companies: Companies = list
        .data_list
        .unwrap_or_default()
        .try_into()
        .expect("Failed to convert");

    db.insert_companies(&companies)
        .await
        .inspect_err(|e| {
            tracing::error!(?e, "Failed to insert companies");
        })
        .unwrap();
}
