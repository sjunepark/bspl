use bspl::AppConfig;
use config::Config;
use db::LibsqlDb;
use smes::get_bspl_htmls;

/// 1. Get all companies from the database
/// 2. Get HTMLs from smes(This is an expensive job)
/// 3. Insert HTMLs into the database
#[tokio::main]
async fn main() {
    tracing_setup::span!("main");

    let config = Config::builder()
        .add_source(config::File::with_name("Settings"))
        .build()
        .expect("Failed to build config");

    let app: AppConfig = config
        .try_deserialize()
        .expect("Failed to deserialize config");

    println!("{:?}", app);

    let db = LibsqlDb::new_local("db/local.db")
        .await
        .expect("Failed to get db");

    let all_ids_to_query = db.get_company_ids().await.expect("Failed to get companies");
    let ids_with_htmls = db.get_html_ids().await.expect("Failed to get html ids");

    let ids_to_query = all_ids_to_query.difference(&ids_with_htmls);
    let ids_already_queried = all_ids_to_query.intersection(&ids_with_htmls);

    // 1. Query new ids
}
