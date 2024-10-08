use db::LibsqlDb;
use smes::{ListApi, ListPayloadBuilder};

#[tokio::main]
async fn main() {
    tracing_setup::span!("main");

    let db = LibsqlDb::new_local("db/local.db")
        .await
        .inspect_err(|e| {
            tracing::error!(?e, "Failed to create db");
        })
        .expect("Failed to create db");

    let mut api = ListApi::new();

    let total_count = api
        .get_company_list_count()
        .await
        .expect("Failed to get total count");

    let payload = ListPayloadBuilder::default()
        .page_size(total_count)
        .build()
        .expect("Failed to build payload");

    let response = api
        .get_company_list(&payload)
        .await
        .expect("Failed to make request");

    let companies: Vec<_> = response
        .data_list
        .expect("data_list is None")
        .into_iter()
        .map(|c| c.try_into().expect("Failed to convert company"))
        .collect();

    db.upsert_companies(companies)
        .await
        .expect("Failed to upsert companies");
}
