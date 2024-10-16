use crate::error::DbError;
use crate::smes::{CompanyDb, HtmlDb};
use hashbrown::HashSet;
use libsql::params::IntoParams;
use model::{company, ModelError};
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use std::fmt::Debug;
use std::future::Future;
use std::path::Path;

pub trait Db: Sized + CompanyDb + HtmlDb {
    fn new<P: AsRef<Path> + Debug>(db_url: P) -> impl Future<Output = Self>;
    fn health_check(&self) -> impl Future<Output = Result<(), DbError>>;
}

pub(crate) trait Params {
    fn params(&self) -> impl IntoParams;
}

// region: PostgresqlDb
pub struct PostgresqlDb {
    pub pool: sqlx::PgPool,
}

impl Db for PostgresqlDb {
    #[tracing::instrument]
    async fn new<P: AsRef<Path> + Debug>(db_url: P) -> Self {
        let connection_string = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");
        let pool = PgPoolOptions::new()
            .connect(&connection_string)
            .await
            .expect("Failed to create connection pool");

        Self { pool }
    }

    #[tracing::instrument(skip(self))]
    async fn health_check(&self) -> Result<(), DbError> {
        sqlx::query!("SELECT 1 AS value")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(Into::into)
    }
}

// region: LibsqlDb
pub struct LibsqlDb {
    pub connection: libsql::Connection,
}

impl Db for LibsqlDb {
    #[tracing::instrument(skip(self))]
    async fn health_check(&self) -> Result<(), DbError> {
        self.connection
            .query("SELECT 1", ())
            .await
            .map(|_| ())
            .map_err(Into::into)
    }

    #[tracing::instrument]
    async fn new<P: AsRef<Path> + Debug>(db_url: P) -> Self {
        let db = libsql::Builder::new_local(db_url)
            .build()
            .await
            .expect("Failed to create connection")
            .connect()
            .expect("Failed to connect");

        Self { connection: db }
    }
}
impl LibsqlDb {
    pub(crate) async fn get_all_from<T: for<'de> Deserialize<'de>>(
        &self,
        table: &str,
    ) -> Result<Vec<T>, DbError> {
        let mut rows = self
            .connection
            .query(&format!("SELECT * from {}", table), ())
            .await?;
        let mut items = Vec::new();

        while let Some(row) = rows.next().await? {
            let item = libsql::de::from_row::<T>(&row)?;
            items.push(item);
        }

        Ok(items)
    }

    pub(crate) async fn get_all_ids_from(
        &self,
        table: &str,
    ) -> Result<HashSet<company::Id>, DbError> {
        let mut rows = self
            .connection
            .query(&format!("SELECT * from {}", table), ())
            .await?;
        let mut ids = HashSet::new();

        #[derive(Deserialize)]
        struct IdStruct {
            smes_id: String,
        }

        while let Some(row) = rows.next().await? {
            let id_struct = libsql::de::from_row::<IdStruct>(&row)?;
            ids.insert(id_struct.smes_id.try_into().map_err(ModelError::from)?);
        }

        Ok(ids)
    }
}

// endregion: LibsqlDb

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn libsql_db_should_connect() {
        tracing_setup::span!("test");

        let db = LibsqlDb::new(":memory:").await;

        let mut rows = db
            .connection
            .query(r#"SELECT "HI""#, ())
            .await
            .inspect_err(|e| tracing::error!(?e))
            .expect("Failed to execute query");

        if let Ok(Some(row)) = rows.next().await {
            let value: String = row.get(0).expect("Failed to get value");
            assert_eq!(value, "HI");
        }
        assert!(rows.next().await.expect("Unable to get row").is_none());
    }
}
