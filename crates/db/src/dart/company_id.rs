use diesel::prelude::*;
use std::future::Future;

use crate::schema::dart::company_id::dsl;
use crate::{model, DbError, PostgresDb};

pub trait CompanyIdDb {
    fn get_company_ids(
        &mut self,
    ) -> impl Future<Output = Result<Vec<model::dart::CompanyId>, DbError>>;
    fn insert_company_ids(
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
        const POSTGRES_MAX_PARAMETERS: usize = 65535;
        const BUFFER_DIVISOR: usize = 100;

        for chunk in company_ids.chunks(POSTGRES_MAX_PARAMETERS / BUFFER_DIVISOR) {
            tracing::trace!(chunk_size = chunk.len(), "Inserting chunk of company_ids");
            self.insert_company_ids_inner(chunk.to_vec()).await?;
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
}

#[cfg(test)]
mod tests {
    use crate::dart::CompanyIdDb;
    use crate::test_utils::{PostgresTestContext, TestContext};

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
}
