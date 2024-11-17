use crate::entities::dart::filing;
use crate::entities::dart::prelude::*;
use crate::{DbError, PostgresDb, POSTGRES_MAX_PARAMETERS};
use sea_orm::EntityTrait;
use std::future::Future;

pub trait FilingDb {
    fn get_filings(&mut self) -> impl Future<Output = Result<Vec<filing::Model>, DbError>>;
    fn insert_filings(
        &mut self,
        filings: Vec<filing::ActiveModel>,
    ) -> impl Future<Output = Result<(), DbError>>;
}

impl FilingDb for PostgresDb {
    #[tracing::instrument(skip(self))]
    async fn get_filings(&mut self) -> Result<Vec<filing::Model>, DbError> {
        Ok(Filing::find().all(&self.conn).await?)
    }

    #[tracing::instrument(skip(self, filings))]
    async fn insert_filings(&mut self, filings: Vec<filing::ActiveModel>) -> Result<(), DbError> {
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
        filings: Vec<filing::ActiveModel>,
    ) -> Result<(), DbError> {
        let total_filing_count = filings.len();
        let res = Filing::insert_many(filings).exec(&self.conn).await?;
        tracing::info!(?res.last_insert_id, "Inserted {} filings", total_filing_count);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::dart::FilingDb;
    use crate::test_utils::{PostgresTestContext, TestContext};

    #[tokio::test]
    async fn insert_and_get_filings_should_work() {
        let function_id = utils::function_id!();
        let mut ctx = PostgresTestContext::new(&function_id).await;

        let ids = (0..10_u64).map(|i| 10000000 + i).collect::<Vec<_>>();
        let mut inserted_filings = ctx.populate_filings(&ids).await;

        let mut selected_filings: Vec<_> =
            ctx.db().get_filings().await.expect("Failed to get filings");

        inserted_filings.sort_by_key(|c| c.dart_id.clone());
        selected_filings.sort_by_key(|c| c.dart_id.clone());

        assert_eq!(inserted_filings, selected_filings,);
    }
}
