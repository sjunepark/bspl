use diesel::prelude::*;
use std::future::Future;

use crate::schema::dart::filing::dsl;
use crate::{model, DbError, PostgresDb, POSTGRES_MAX_PARAMETERS};

pub trait FilingDb {
    fn get_filings(&mut self) -> impl Future<Output = Result<Vec<model::dart::Filing>, DbError>>;
    fn insert_filings(
        &mut self,
        filings: Vec<model::dart::NewFiling>,
    ) -> impl Future<Output = Result<(), DbError>>;
}

impl FilingDb for PostgresDb {
    #[tracing::instrument(skip(self))]
    async fn get_filings(&mut self) -> Result<Vec<model::dart::Filing>, DbError> {
        Ok(dsl::filing.load(&mut self.conn)?)
    }

    #[tracing::instrument(skip(self, filings))]
    async fn insert_filings(
        &mut self,
        filings: Vec<model::dart::NewFiling>,
    ) -> Result<(), DbError> {
        const BUFFER_DIVISOR: usize = 100;

        for chunk in filings.chunks(POSTGRES_MAX_PARAMETERS / BUFFER_DIVISOR) {
            tracing::trace!(chunk_size = chunk.len(), "Inserting chunk of filings");
            self.insert_filings_inner(chunk.to_vec()).await?;
        }
        Ok(())
    }
}

impl PostgresDb {
    async fn insert_filings_inner(
        &mut self,
        filings: Vec<model::dart::NewFiling>,
    ) -> Result<(), DbError> {
        let total_filing_count = filings.len();

        self.conn.transaction(|conn| {
            let insert_count = diesel::insert_into(dsl::filing)
                .values(&filings)
                .execute(conn)?;

            if insert_count == total_filing_count {
                tracing::trace!("Inserted {} filings", insert_count);
                Ok(())
            } else {
                tracing::error!(
                    "Inserted {}/{} filings. Rolling back transaction",
                    insert_count,
                    total_filing_count
                );
                Err(diesel::result::Error::RollbackTransaction)
            }
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::dart::FilingDb;
    use crate::model::dart::NewFiling;
    use crate::test_utils::{PostgresTestContext, TestContext};

    #[tokio::test]
    async fn insert_and_get_filings_should_work() {
        let function_id = utils::function_id!();
        let mut ctx = PostgresTestContext::new(&function_id).await;

        let ids = (0..10_u64).map(|i| 10000000 + i).collect::<Vec<_>>();
        let mut inserted_filings = ctx.populate_filings(&ids).await;

        let mut selected_filings: Vec<_> = ctx
            .db()
            .get_filings()
            .await
            .expect("Failed to get filings")
            .into_iter()
            .map(NewFiling::from)
            .collect();

        inserted_filings.sort_by_key(|c| c.dart_id.clone());
        selected_filings.sort_by_key(|c| c.dart_id.clone());

        assert_eq!(inserted_filings, selected_filings,);
    }
}
