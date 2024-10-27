use crate::error::DbError;
use crate::{dart, smes};

use diesel::prelude::*;
use diesel::sql_query;
use std::fmt::Debug;
use std::future::Future;
use std::path::Path;

pub trait Db: Sized + smes::CompanyDb + smes::HtmlDb + dart::FilingDb {
    fn new<P: AsRef<Path> + Debug>(db_url: P) -> impl Future<Output = Self>;
    fn health_check(&mut self) -> impl Future<Output = Result<(), DbError>>;
}

// region: Postgres
pub struct PostgresDb {
    pub conn: PgConnection,
}

impl Db for PostgresDb {
    #[tracing::instrument]
    async fn new<P: AsRef<Path> + Debug>(connection_string: P) -> Self {
        let connection_string = connection_string.as_ref().to_string_lossy();
        let conn = PgConnection::establish(&connection_string)
            .expect("Failed to establish connection to db");

        Self { conn }
    }

    #[tracing::instrument(skip(self))]
    async fn health_check(&mut self) -> Result<(), DbError> {
        sql_query("SELECT 1").execute(&mut self.conn)?;
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
        let mut ctx = PostgresTestContext::new(&function_id).await;
        assert!(ctx.db().health_check().await.is_ok());
    }
}
