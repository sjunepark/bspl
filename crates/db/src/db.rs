use crate::error::DbError;
use libsql::params::IntoParams;
use serde::Deserialize;
use std::fmt::Debug;
use std::path::Path;

pub struct LibsqlDb {
    pub connection: libsql::Connection,
}

impl LibsqlDb {
    #[tracing::instrument(skip(self))]
    pub async fn health_check(&self) -> Result<(), DbError> {
        self.connection
            .query("SELECT 1", ())
            .await
            .map(|_| ())
            .map_err(Into::into)
    }

    #[tracing::instrument]
    pub async fn new_local<P: AsRef<Path> + Debug>(db_url: P) -> Result<Self, DbError> {
        let db = libsql::Builder::new_local(db_url)
            .build()
            .await?
            .connect()?;

        Ok(Self { connection: db })
    }

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn db_should_connect() {
        tracing_setup::span!("test");

        let db = LibsqlDb::new_local(":memory:")
            .await
            .inspect_err(|e| tracing::error!(?e, "Failed to create connection"))
            .unwrap();

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

pub(crate) trait Params {
    fn params(&self) -> impl IntoParams;
}
