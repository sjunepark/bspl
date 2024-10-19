use db::smes::CompanyDb;
use db::{Db, PostgresDb};

#[tokio::main]
async fn main() {
    tracing_setup::span!("main");

    let json = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/resources/list.json"
    ))
    .inspect_err(|e| tracing::error!(?e, "Failed to read file"))
    .unwrap();

    let response = serde_json::from_str::<smes::ListResponse>(&json)
        .inspect_err(|e| tracing::error!(?e, "Failed to deserialize"))
        .unwrap();

    let connection_string = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let db = PostgresDb::new(connection_string).await;

    let companies: Vec<model::table::Company> = response
        .companies()
        .unwrap_or_default()
        .into_iter()
        .collect();

    db.insert_companies(companies)
        .await
        .inspect_err(|e| {
            tracing::error!(?e, "Failed to insert companies");
        })
        .unwrap();
}
