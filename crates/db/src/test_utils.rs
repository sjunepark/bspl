#![cfg(test)]

mod postgres;

use crate::db::Db;
use fake::{Fake, Faker};
use sea_orm::{Set, TryIntoModel};
use tokio::sync::mpsc;

use crate::entities::dart::company_id;
use crate::model::smes::NewHtml;
pub(crate) use postgres::PostgresTestContext;

pub(crate) trait TestContext<D: Db> {
    async fn new(function_id: &str) -> Self;
    fn db(&mut self) -> &mut D;

    /// Populate the database with fake companies.
    #[tracing::instrument(skip(self))]
    async fn populate_companies(&mut self, ids: &[u64]) -> Vec<crate::model::smes::NewCompany> {
        let new_companies: Vec<crate::model::smes::NewCompany> = ids
            .iter()
            .map(|id| {
                let company = Faker.fake::<crate::model::smes::Company>();
                crate::model::smes::NewCompany {
                    smes_id: id
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
            .expect("Failed to insert companies");

        new_companies
    }

    /// Populate the database with fake HTMLs.
    ///
    /// ## Warning
    /// To satisfy the foreign key constraint, the Company table will be populated first.
    #[tracing::instrument(skip(self))]
    async fn populate_htmls(&mut self, ids: &[u64]) -> Vec<NewHtml> {
        self.populate_companies(ids).await;

        let htmls: Vec<NewHtml> =
            ids.iter()
                .map(|id| {
                    let html = Faker.fake::<NewHtml>();
                    NewHtml {
                        smes_id: id.to_string().as_str().try_into().expect(
                            "dummy creation logic needs to be fixed within the source code",
                        ),
                        ..html
                    }
                })
                .collect();

        let (tx, rx) = mpsc::unbounded_channel();

        for html in &htmls {
            tracing::trace!(?html, "Sending HTML");
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

    #[tracing::instrument(skip(self))]
    async fn populate_filings(&mut self, ids: &[u64]) -> Vec<crate::model::dart::NewFiling> {
        let new_filings: Vec<crate::model::dart::NewFiling> = ids
            .iter()
            .map(|id| {
                let filing = Faker.fake::<crate::model::dart::NewFiling>();
                crate::model::dart::NewFiling {
                    dart_id: id
                        .to_string()
                        .as_str()
                        .try_into()
                        .expect("failed to create dummy dart_id"),
                    report_name: filing.report_name,
                    receipt_number: filing.receipt_number,
                    filer_name: filing.filer_name,
                    receipt_date: filing.receipt_date,
                    remark: filing.remark,
                }
            })
            .collect();

        self.db()
            .insert_filings(new_filings.clone())
            .await
            .expect("Failed to insert filings");

        new_filings
    }

    #[tracing::instrument(skip(self))]
    async fn populate_company_ids(&mut self, ids: &[u64]) -> Vec<company_id::Model> {
        let new_company_ids: Vec<company_id::ActiveModel> = ids
            .iter()
            .map(|id| {
                let company_id = Faker.fake::<company_id::ActiveModel>();
                company_id::ActiveModel {
                    dart_id: Set(id
                        .to_string()
                        .as_str()
                        .try_into()
                        .expect("failed to create dummy company_id")),
                    ..company_id
                }
            })
            .collect();

        self.db()
            .insert_company_ids(new_company_ids.clone())
            .await
            .expect("Failed to insert company_ids");

        new_company_ids
            .into_iter()
            .map(|c| c.try_into_model().expect("failed to convert to model"))
            .collect()
    }
}
