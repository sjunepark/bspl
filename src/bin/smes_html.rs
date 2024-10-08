use db::LibsqlDb;
use model::company;

/// 1. Get all companies from the database
/// 2. Get htmls from smes
/// 3. Insert htmls into the database
#[tokio::main]
async fn main() {
    tracing_setup::span!("main");

    let db = LibsqlDb::new_local("db/local.db")
        .await
        .expect("Failed to create db");

    let ids: Vec<company::Id> = db
        .get_companies()
        .await
        .expect("Failed to get companies")
        .into_iter()
        .map(|c| c.smes_id.try_into().expect("Failed to convert id"))
        .collect::<Vec<_>>();

    let htmls = smes::get_bspl_htmls(ids).await;
}
