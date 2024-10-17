#![cfg(test)]

mod libsql;
mod postgres;

use crate::db::Db;
use fake::{Fake, Faker};
use model::table;
use tokio::sync::mpsc;

pub(crate) use libsql::LibsqlTestContext;
pub(crate) use postgres::PostgresTestContext;

pub(crate) trait TestContext<D: Db> {
    async fn new(function_id: &str) -> Self;
    fn db(&self) -> &D;

    /// Populate the database with fake companies.
    async fn populate_companies(&self, ids: &[u64]) -> Vec<table::Company> {
        let companies: Vec<table::Company> = ids
            .iter()
            .map(|id| {
                let company = Faker.fake::<table::Company>();
                table::Company {
                    smes_id: id
                        .to_string()
                        .try_into()
                        .expect("failed to create proper dummy smes_id"),
                    ..company
                }
            })
            .collect();

        self.db()
            .insert_companies(companies.clone())
            .await
            .inspect_err(|e| {
                tracing::error!(?e, "Failed to insert companies");
            })
            .unwrap();

        companies
    }

    /// Populate the database with fake HTMLs.
    ///
    /// ## Warning
    /// To satisfy the foreign key constraint, the Company table will be populated first.
    async fn populate_htmls(&self, ids: &[u64]) -> Vec<table::Html> {
        self.populate_companies(ids).await;

        let htmls: Vec<table::Html> = ids
            .iter()
            .map(|id| {
                let html = Faker.fake::<table::Html>();
                table::Html {
                    smes_id: id
                        .to_string()
                        .try_into()
                        .expect("failed to create dummy smes_id"),
                    ..html
                }
            })
            .collect();

        let (tx, rx) = mpsc::unbounded_channel();
        for html in &htmls {
            tx.send(html.clone()).expect("Failed to send HTML");
        }
        drop(tx);

        self.db()
            .insert_html_channel(rx)
            .await
            .inspect_err(|e| {
                tracing::error!(?e, "Failed to insert HTMLs");
            })
            .expect("Failed to insert HTMLs");

        htmls
    }
}
