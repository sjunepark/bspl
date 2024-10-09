use bspl::AppConfig;
use config::Config;
use db::LibsqlDb;
use smes::get_bspl_htmls;
use tracing::Instrument;

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

    let db = LibsqlDb::new_local("db/local.db")
        .in_current_span()
        .await
        .expect("Failed to get db");

    // 1. Get all companies from the database
    let all_ids_to_query = db
        .get_company_ids()
        .in_current_span()
        .await
        .expect("Failed to get companies");
    let ids_with_htmls = db
        .get_html_ids()
        .in_current_span()
        .await
        .expect("Failed to get html ids");

    let ids_to_query = all_ids_to_query.difference(&ids_with_htmls).cloned();
    let ids_already_queried = all_ids_to_query.intersection(&ids_with_htmls);

    // 2. Get HTMLs from smes
    let new_htmls = get_bspl_htmls(ids_to_query.collect())
        .in_current_span()
        .await;

    // 3-1.
    // Insert new HTMLs to the database
    // We're calling `insert_htmls` rather than `upsert_htmls`
    // because we're sure that the HTMLs are new.
    // Or else, it means that an invariant has happened.
    db.insert_htmls(new_htmls)
        .in_current_span()
        .await
        .expect("Failed to upsert htmls");

    if app.update_all_html {
        // 3-2. Update all HTMLs
        let htmls = get_bspl_htmls(ids_already_queried.cloned().collect())
            .in_current_span()
            .await;
        db.upsert_htmls(htmls)
            .in_current_span()
            .await
            .expect("Failed to upsert htmls");
    }
}
