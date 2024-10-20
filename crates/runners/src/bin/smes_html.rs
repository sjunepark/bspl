use db::smes::{CompanyDb, HtmlDb};
use db::{Db, PostgresDb};
use figment::providers::{Format, Toml};
use figment::Figment;
use runners::AppConfig;
use smes::get_bspl_htmls;
use tracing::Instrument;

#[tokio::main]
async fn main() {
    tracing_setup::span!("main");

    let app: AppConfig = Figment::new()
        .merge(Toml::file("Settings.toml"))
        .extract()
        .expect("Failed to load settings");

    let connection_string = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let mut db = PostgresDb::new(connection_string).in_current_span().await;

    // 1. Get all companies from the database
    let all_ids_to_query = db
        .get_company_ids()
        .in_current_span()
        .await
        .expect("Failed to get companies");
    let ids_with_htmls = db
        .select_html_ids()
        .in_current_span()
        .await
        .expect("Failed to get html ids");

    tracing::info!(?ids_with_htmls, "These queried ids will be skipped");

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
    db.insert_html_channel(new_htmls)
        .in_current_span()
        .await
        .expect("Failed to upsert htmls");

    if app.update_all_html {
        // 3-2. Update all HTMLs
        let htmls = get_bspl_htmls(ids_already_queried.cloned().collect())
            .in_current_span()
            .await;
        db.upsert_html_channel(htmls)
            .in_current_span()
            .await
            .expect("Failed to upsert htmls");
    }
}

#[cfg(test)]
mod tests {
    use hashbrown::HashSet;
    use model::company;

    #[test]
    fn hashset_difference_should_work_as_expected() {
        let set1: HashSet<company::Id> = [1000000, 2000000, 3000000]
            .into_iter()
            .map(|id| {
                company::Id::try_new(id.to_string().as_str()).expect("Failed to create company id")
            })
            .collect();
        let set2: HashSet<company::Id> = [2000000, 3000000, 4000000]
            .into_iter()
            .map(|id| {
                company::Id::try_new(id.to_string().as_str()).expect("Failed to create company id")
            })
            .collect();

        let difference: HashSet<company::Id> = set1.difference(&set2).cloned().collect();

        let expected: HashSet<company::Id> = [1000000]
            .into_iter()
            .map(|id| {
                company::Id::try_new(id.to_string().as_str()).expect("Failed to create company id")
            })
            .collect();

        assert_eq!(difference, expected);
    }
}
