#![cfg(test)]

mod postgres;

use crate::db::Db;
use fake::{Fake, Faker};
use tokio::sync::mpsc;

use crate::model::smes::NewHtml;
pub(crate) use postgres::PostgresTestContext;

pub(crate) trait TestContext<D: Db> {
    async fn new(function_id: &str) -> Self;
    fn db(&mut self) -> &mut D;

    /// Populate the database with fake companies.
    async fn populate_companies(&mut self, ids: &[u64]) -> Vec<crate::model::smes::NewCompany> {
        let new_companies: Vec<crate::model::smes::NewCompany> = ids
            .iter()
            .map(|id| {
                let company = Faker.fake::<crate::model::smes::Company>();
                crate::model::smes::NewCompany {
                    company_id: id
                        .to_string()
                        .as_str()
                        .try_into()
                        .expect("failed to create dummy company_id"),
                    representative_name: company.representative_name,
                    headquarters_address: company.headquarters_address,
                    business_registration_number: company.business_registration_number,
                    company_name: company.company_name,
                    industry_code: company.industry_code,
                    industry_name: company.industry_name,
                }
            })
            .collect();

        self.db()
            .insert_companies(new_companies.clone())
            .await
            .inspect_err(|e| {
                tracing::error!(?e, "Failed to insert companies");
            })
            .unwrap();

        new_companies
    }

    /// Populate the database with fake HTMLs.
    ///
    /// ## Warning
    /// To satisfy the foreign key constraint, the Company table will be populated first.
    async fn populate_htmls(&mut self, ids: &[u64]) -> Vec<NewHtml> {
        self.populate_companies(ids).await;

        let htmls: Vec<NewHtml> = ids
            .iter()
            .map(|id| {
                let html = Faker.fake::<NewHtml>();
                NewHtml {
                    company_id: id.to_string(),
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
