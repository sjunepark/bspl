use crate::error::DbError;
use crate::smes::{CompanyDb, HtmlDb};
use sqlx::postgres::PgPoolOptions;
use std::fmt::Debug;
use std::future::Future;
use std::path::Path;

pub trait Db: Sized + CompanyDb + HtmlDb {
    fn new<P: AsRef<Path> + Debug>(db_url: P) -> impl Future<Output = Self>;
    fn health_check(&self) -> impl Future<Output = Result<(), DbError>>;
}

// region: Postgres
pub struct PostgresDb {
    pub pool: sqlx::PgPool,
}

impl Db for PostgresDb {
    #[tracing::instrument]
    async fn new<P: AsRef<Path> + Debug>(connection_string: P) -> Self {
        let connection_string = connection_string.as_ref().to_string_lossy();
        // <https://github.com/rust10x/rust-web-app/blob/main/crates/libs/lib-core/src/model/store/mod.rs>
        // This is not an ideal situation; however, with sqlx 0.7.1, when executing 'cargo test', some tests that use sqlx fail at a
        // rather low level (in the tokio scheduler). It appears to be a low-level thread/async issue, as removing/adding
        // tests causes different tests to fail. The cause remains uncertain, but setting max_connections to 1 resolves the issue.
        // The good news is that max_connections still function normally for a regular run.
        // This issue is likely due to the unique requirements unit tests impose on their execution, and therefore,
        // while not ideal, it should serve as an acceptable temporary solution.
        // It's a very challenging issue to investigate and narrow down. The alternative would have been to stick with sqlx 0.6.x, which
        // is potentially less ideal and might lead to confusion as to why we are maintaining the older version in this blueprint.
        let max_connections = if cfg!(test) { 1 } else { 5 };

        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(&connection_string)
            .await
            .expect("Failed to create connection pool");

        Self { pool }
    }

    #[tracing::instrument(skip(self))]
    async fn health_check(&self) -> Result<(), DbError> {
        let result = sqlx::query!("SELECT 1 AS value")
            .fetch_one(&self.pool)
            .await?;

        tracing::trace!(?result);
        tracing::trace!(value = ?result.value);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{PostgresTestContext, TestContext};

    #[tokio::test]
    async fn postgres_health_check() {
        tracing_setup::span!("test");
        let function_id = utils::function_id!();
        let ctx = PostgresTestContext::new(&function_id).await;
        assert!(ctx.db().health_check().await.is_ok());
    }
}
