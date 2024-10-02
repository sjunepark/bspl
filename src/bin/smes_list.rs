use db::{Companies, LibsqlDb};
use smes::{ListApi, ListPayloadBuilder};

#[tokio::main]
async fn main() {
    tracing_setup::subscribe();

    let db = LibsqlDb::new_local("db/local.db")
        .await
        .inspect_err(|e| {
            tracing::error!(?e, "Failed to create db");
        })
        .expect("Failed to create db");

    let api = ListApi::new();

    let total_count = api
        .get_total_count()
        .await
        .expect("Failed to get total count");

    let payload = ListPayloadBuilder::default()
        .page_size(total_count)
        .build()
        .expect("Failed to build payload");

    let response = api
        .make_request(&payload)
        .await
        .expect("Failed to make request");

    let companies: Companies = response
        .data_list
        .expect("data_list is None")
        .try_into()
        .expect("Failed to convert data_list to Companies");

    db.upsert_companies(&companies)
        .await
        .expect("Failed to upsert companies");
}
