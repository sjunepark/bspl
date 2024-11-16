use crate::{DbError, PostgresDb, POSTGRES_MAX_PARAMETERS};

use crate::schema::smes::company::dsl;
use diesel::prelude::*;
use diesel::upsert::excluded;
use hashbrown::HashSet;
use std::future::Future;
use types::company;

pub trait CompanyDb {
    fn get_companies(
        &mut self,
    ) -> impl Future<Output = Result<Vec<crate::model::smes::Company>, DbError>>;
    fn get_smes_ids(&mut self) -> impl Future<Output = Result<HashSet<company::SmesId>, DbError>>;
    fn insert_companies(
        &mut self,
        companies: Vec<crate::model::smes::NewCompany>,
    ) -> impl Future<Output = Result<(), DbError>>;
    fn upsert_companies(
        &mut self,
        companies: Vec<crate::model::smes::NewCompany>,
    ) -> impl Future<Output = Result<(), DbError>>;
}

impl CompanyDb for PostgresDb {
    async fn get_companies(&mut self) -> Result<Vec<crate::model::smes::Company>, DbError> {
        Ok(dsl::company.load(&mut self.diesel_conn)?)
    }

    async fn get_smes_ids(&mut self) -> Result<HashSet<company::SmesId>, DbError> {
        dsl::company
            .select(dsl::smes_id)
            .load::<String>(&mut self.diesel_conn)?
            .into_iter()
            .map(|id| company::SmesId::try_from(id.as_str()).map_err(DbError::from))
            .collect::<Result<HashSet<_>, _>>()
    }

    #[tracing::instrument(skip(self, companies))]
    async fn insert_companies(
        &mut self,
        companies: Vec<crate::model::smes::NewCompany>,
    ) -> Result<(), DbError> {
        // A single INSERT statement can have more than 1 binding,
        // as so we're putting a generous buffer.
        // If you use a small divisor(ex. 4), it could fail due to multiple bindings.
        const BUFFER_DIVISOR: usize = 100;

        for chunk in companies.chunks(POSTGRES_MAX_PARAMETERS / BUFFER_DIVISOR) {
            tracing::trace!(chunk_size = chunk.len(), "Inserting chunk of companies");
            self.insert_companies_inner(chunk.to_vec()).await?;
        }
        Ok(())
    }

    #[tracing::instrument(skip(self, companies))]
    async fn upsert_companies(
        &mut self,
        companies: Vec<crate::model::smes::NewCompany>,
    ) -> Result<(), DbError> {
        // A single INSERT statement can have more than 1 binding,
        // as so we're putting a generous buffer.
        // If you use a small divisor(ex. 4), it could fail due to multiple bindings.
        const BUFFER_DIVISOR: usize = 100;

        for chunk in companies.chunks(POSTGRES_MAX_PARAMETERS / BUFFER_DIVISOR) {
            tracing::trace!(chunk_size = chunk.len(), "Upserting chunk of companies");
            self.upsert_companies_inner(chunk.to_vec()).await?;
        }
        Ok(())
    }
}

impl PostgresDb {
    #[tracing::instrument(skip(self, companies))]
    async fn insert_companies_inner(
        &mut self,
        companies: Vec<crate::model::smes::NewCompany>,
    ) -> Result<(), DbError> {
        let total_company_count = companies.len() as u64;

        self.diesel_conn.transaction::<(), _, _>(|conn| {
            let insert_count = diesel::insert_into(dsl::company)
                .values(&companies)
                .execute(conn)?;

            if insert_count == total_company_count as usize {
                tracing::trace!("Inserted {} companies", insert_count);
                Ok(())
            } else {
                tracing::error!(
                    "Inserted {}/{} companies. Rolling back transaction",
                    insert_count,
                    total_company_count
                );
                Err(diesel::result::Error::RollbackTransaction)
            }
        })?;
        Ok(())
    }

    #[tracing::instrument(skip(self, companies))]
    async fn upsert_companies_inner(
        &mut self,
        companies: Vec<crate::model::smes::NewCompany>,
    ) -> Result<(), DbError> {
        self.diesel_conn.transaction(|conn| {
            let insert_count = diesel::insert_into(dsl::company)
                .values(&companies)
                .on_conflict(dsl::smes_id)
                .do_update()
                .set((
                    dsl::representative_name.eq(excluded(dsl::representative_name)),
                    dsl::headquarters_address.eq(excluded(dsl::headquarters_address)),
                    dsl::business_registration_number
                        .eq(excluded(dsl::business_registration_number)),
                    dsl::company_name.eq(excluded(dsl::company_name)),
                    dsl::industry_code.eq(excluded(dsl::industry_code)),
                    dsl::industry_name.eq(excluded(dsl::industry_name)),
                ))
                .execute(conn)?;

            if insert_count == companies.len() {
                tracing::trace!("Upserted {} companies", insert_count);
                Ok(())
            } else {
                tracing::error!(
                    "Upserted {}/{} companies. Rolling back transaction",
                    insert_count,
                    companies.len()
                );
                Err(diesel::result::Error::RollbackTransaction)
            }
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::model::smes::NewCompany;
    use crate::smes::company::CompanyDb;
    use crate::test_utils::{PostgresTestContext, TestContext};
    use fake::Fake;
    use hashbrown::HashSet;

    #[tokio::test]
    async fn insert_and_get_companies_should_work() {
        let function_id = utils::function_id!();
        let mut ctx = PostgresTestContext::new(&function_id).await;

        let ids = (0..10_u64).map(|i| 1000000 + i).collect::<Vec<_>>();
        let mut inserted_companies = ctx.populate_companies(&ids).await;

        let mut selected_companies: Vec<_> = ctx
            .db()
            .get_companies()
            .await
            .expect("Failed to get companies")
            .into_iter()
            .map(NewCompany::from)
            .collect();

        inserted_companies.sort_by_key(|c| c.smes_id.clone());
        selected_companies.sort_by_key(|c| c.smes_id.clone());

        assert_eq!(inserted_companies, selected_companies,);
    }

    #[tokio::test]
    async fn get_smes_ids_should_work() {
        let function_id = utils::function_id!();
        let mut ctx = PostgresTestContext::new(&function_id).await;

        let ids = (0..10_u64).map(|i| 1000000 + i).collect::<Vec<_>>();
        let inserted_ids: HashSet<_> = ctx
            .populate_companies(&ids)
            .await
            .into_iter()
            .map(|company| company.smes_id)
            .collect();

        let selected_ids = ctx
            .db()
            .get_smes_ids()
            .await
            .expect("Failed to get company ids");

        assert_eq!(inserted_ids, selected_ids);
    }

    #[tokio::test]
    async fn upsert_companies_should_work() {
        // region: Arrange
        tracing_setup::span!("test");

        let function_id = utils::function_id!();
        let mut ctx = PostgresTestContext::new(&function_id).await;

        // Set up basic companies
        let ids = (0..10_u64).map(|i| 1000000 + i).collect::<Vec<_>>();
        let companies = ctx.populate_companies(&ids).await;

        // Create companies to upsert: from existing companies.
        const UPDATED_REPRESENTATIVE_NAME: &str = "Updated";
        let mut updated_companies = companies
            .iter()
            .map(|c| NewCompany {
                representative_name: UPDATED_REPRESENTATIVE_NAME.to_string(),
                ..c.clone()
            })
            .collect::<Vec<_>>();

        // Add a new company to see that this company was properly updated
        let mut new_company = ().fake::<NewCompany>();
        const NEW_COMPANY_ID: &str = "2000000";
        new_company.smes_id = NEW_COMPANY_ID
            .try_into()
            .expect("failed to create dummy smes_id");
        let new_company_representative_name = new_company.representative_name.clone();
        updated_companies.push(new_company);

        // Remove a company to check that this company was not updated
        let removed_company = updated_companies.pop().unwrap();
        let removed_smes_id = removed_company.smes_id;
        // endregion: Arrange

        // region: Action
        let db = ctx.db();
        db.upsert_companies(updated_companies)
            .await
            .inspect_err(|e| tracing::error!(?e, "Failed to upsert companies"))
            .unwrap();
        // endregion: Action

        // region: Assert
        let db_companies = db
            .get_companies()
            .await
            .inspect_err(|e| tracing::error!(?e, "Failed to get companies"))
            .unwrap();

        for company in &db_companies {
            match company.smes_id.as_ref().as_str() {
                NEW_COMPANY_ID => {
                    assert_eq!(company.representative_name, new_company_representative_name);
                }
                id if id == removed_smes_id.as_ref() => {
                    // Not upserted company name should not change
                    assert_eq!(
                        company.representative_name,
                        removed_company.representative_name
                    );
                }
                _ => {
                    assert_eq!(
                        company.representative_name.as_str(),
                        UPDATED_REPRESENTATIVE_NAME
                    );
                }
            }
        }
        // endregion: Assert
    }
}
