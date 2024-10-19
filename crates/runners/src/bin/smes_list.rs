use db::smes::CompanyDb;
use db::{Db, PostgresDb};
use smes::{ListApi, ListPayloadBuilder};
use tracing::Instrument;

#[tokio::main]
async fn main() {
    tracing_setup::span!("main");

    let connection_string = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let db = PostgresDb::new(connection_string).await;

    let mut api = ListApi::new();

    let total_count = api
        .get_company_list_count()
        .in_current_span()
        .await
        .expect("Failed to get total count");

    let payload = ListPayloadBuilder::default()
        .page_size(total_count)
        .build()
        .expect("Failed to build payload");

    let response = api
        .get_company_list(&payload)
        .in_current_span()
        .await
        .expect("Failed to make request");

    let companies: Vec<_> = response
        .companies()
        .expect("Failed to get companies")
        .into_iter()
        .collect();

    db.upsert_companies(companies)
        .in_current_span()
        .await
        .expect("Failed to upsert companies");
}
