use diesel::prelude::*;
use diesel::upsert::excluded;
use std::future::Future;

use crate::schema::dart::company_id::dsl;
use crate::{model, DbError, PostgresDb, POSTGRES_MAX_PARAMETERS};

pub trait CompanyIdDb {
    fn get_company_ids(
        &mut self,
    ) -> impl Future<Output = Result<Vec<model::dart::CompanyId>, DbError>>;
    fn insert_company_ids(
        &mut self,
        company_ids: Vec<model::dart::CompanyId>,
    ) -> impl Future<Output = Result<(), DbError>>;
    fn upsert_company_ids(
        &mut self,
        company_ids: Vec<model::dart::CompanyId>,
    ) -> impl Future<Output = Result<(), DbError>>;
}

impl CompanyIdDb for PostgresDb {
    #[tracing::instrument(skip(self))]
    async fn get_company_ids(&mut self) -> Result<Vec<model::dart::CompanyId>, DbError> {
        Ok(dsl::company_id.load(&mut self.conn)?)
    }

    #[tracing::instrument(skip(self, company_ids))]
    async fn insert_company_ids(
        &mut self,
        company_ids: Vec<model::dart::CompanyId>,
    ) -> Result<(), DbError> {
        const BUFFER_DIVISOR: usize = 100;

        for chunk in company_ids.chunks(POSTGRES_MAX_PARAMETERS / BUFFER_DIVISOR) {
            tracing::trace!(chunk_size = chunk.len(), "Inserting chunk of company_ids");
            self.insert_company_ids_inner(chunk.to_vec()).await?;
        }
        Ok(())
    }

    #[tracing::instrument(skip(self, company_ids))]
    async fn upsert_company_ids(
        &mut self,
        company_ids: Vec<model::dart::CompanyId>,
    ) -> Result<(), DbError> {
        const BUFFER_DIVISOR: usize = 100;

        for chunk in company_ids.chunks(POSTGRES_MAX_PARAMETERS / BUFFER_DIVISOR) {
            tracing::trace!(chunk_size = chunk.len(), "Upserting chunk of company_ids");
            self.upsert_company_ids_inner(chunk.to_vec()).await?;
        }
        Ok(())
    }
}

impl PostgresDb {
    async fn insert_company_ids_inner(
        &mut self,
        company_ids: Vec<model::dart::CompanyId>,
    ) -> Result<(), DbError> {
        let total_company_id_count = company_ids.len();

        self.conn.transaction(|conn| {
            let insert_count = diesel::insert_into(dsl::company_id)
                .values(&company_ids)
                .execute(conn)?;

            if insert_count == total_company_id_count {
                tracing::trace!("Inserted {} company_ids", insert_count);
                Ok(())
            } else {
                tracing::error!(
                    "Inserted {}/{} company_ids. Rolling back transaction",
                    insert_count,
                    total_company_id_count
                );
                Err(diesel::result::Error::RollbackTransaction)
            }
        })?;
        Ok(())
    }

    async fn upsert_company_ids_inner(
        &mut self,
        company_ids: Vec<model::dart::CompanyId>,
    ) -> Result<(), DbError> {
        let total_company_id_count = company_ids.len();

        self.conn.transaction(|conn| {
            for company_id in &company_ids {
                let insert_count = diesel::insert_into(dsl::company_id)
                    .values(company_id)
                    .on_conflict(dsl::dart_id)
                    .do_update()
                    .set((
                        dsl::company_name.eq(excluded(dsl::company_name)),
                        dsl::stock_code.eq(excluded(dsl::stock_code)),
                        dsl::id_modify_date.eq(excluded(dsl::id_modify_date)),
                    ))
                    .execute(conn)?;

                if insert_count != 1 {
                    tracing::error!(
                        "Upserted {}/{} company_ids. Rolling back transaction",
                        insert_count,
                        total_company_id_count
                    );
                    return Err(diesel::result::Error::RollbackTransaction);
                }
            }
            Ok(())
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::dart::CompanyIdDb;
    use crate::model::dart::CompanyId;
    use crate::test_utils::{PostgresTestContext, TestContext};
    use fake::Fake;

    #[tokio::test]
    async fn insert_and_get_company_ids_should_work() {
        let function_id = utils::function_id!();
        let mut ctx = PostgresTestContext::new(&function_id).await;

        let ids = (0..10_u64).map(|i| 10000000 + i).collect::<Vec<_>>();
        let mut inserted_company_ids = ctx.populate_company_ids(&ids).await;

        let mut selected_company_ids: Vec<_> = ctx
            .db()
            .get_company_ids()
            .await
            .expect("Failed to get company_ids")
            .into_iter()
            .collect();

        inserted_company_ids.sort_by_key(|c| c.dart_id.clone());
        selected_company_ids.sort_by_key(|c| c.dart_id.clone());

        assert_eq!(inserted_company_ids, selected_company_ids,);
    }

    #[tokio::test]
    async fn upsert_company_ids_should_work() {
        tracing_setup::span!("test");

        let function_id = utils::function_id!();
        let mut ctx = PostgresTestContext::new(&function_id).await;

        let ids = (0..10_u64).map(|i| 10000000 + i).collect::<Vec<_>>();
        let mut inserted_company_ids = ctx.populate_company_ids(&ids).await;

        // Create companies to upsert: from existing companies.
        const UPDATED_COMPANY_NAME: &str = "Updated";
        let updated_company_ids = inserted_company_ids
            .iter()
            .map(|company_id| CompanyId {
                company_name: UPDATED_COMPANY_NAME.try_into().expect("Failed to convert"),
                ..company_id.clone()
            })
            .collect::<Vec<_>>();

        // Add a new company to see that this company was properly updated
        let new_company_id = ().fake::<CompanyId>();
        const NEW_DART_ID: &str = "20000000";
        new_company_id
            .dart_id
            .try_into()
            .expect("Failed to convert");
        let new_company_name = new_company_id.company_name.clone();
        updated_company_ids.push(new_company_id.clone());

        let mut selected_company_ids: Vec<_> = ctx
            .db()
            .get_company_ids()
            .await
            .expect("Failed to get company_ids")
            .into_iter()
            .collect();

        inserted_company_ids.sort_by_key(|c| c.dart_id.clone());
        selected_company_ids.sort_by_key(|c| c.dart_id.clone());

        assert_eq!(inserted_company_ids, selected_company_ids,);
    }
}
