use crate::test_utils::TestContext;
use crate::PostgresDb;
use diesel::{Connection, PgConnection};
use sea_orm::Database;
use sea_orm_migration::prelude::*;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::{ContainerAsync, ImageExt};

pub(crate) struct PostgresTestContext {
    db: PostgresDb,
    _node: ContainerAsync<Postgres>,
}

impl TestContext<PostgresDb> for PostgresTestContext {
    #[tracing::instrument]
    async fn new(_function_id: &str) -> Self {
        tracing::trace!("Starting Postgres container");
        let node = Postgres::default()
            .with_db_name("bspl")
            .with_tag("16")
            .start()
            .await
            .expect("Failed to start container");

        let connection_string = format!(
            "postgres://postgres:postgres@127.0.0.1:{}/bspl",
            node.get_host_port_ipv4(5432)
                .await
                .expect("Failed to get port for test db connection")
        );
        tracing::trace!(%connection_string, "Connection string");
        let diesel_conn = PgConnection::establish(&connection_string)
            .expect("Failed to establish connection to test db");

        // Run migrations via SeaORM
        tracing::trace!("Running migrations");
        let db = Database::connect(connection_string)
            .await
            .expect("Failed to connect to db");
        let connection = db.into_schema_manager_connection();

        migration::Migrator::refresh(connection)
            .await
            .expect("Failed to refresh db");

        let db = PostgresDb { diesel_conn };

        Self { db, _node: node }
    }

    fn db(&mut self) -> &mut PostgresDb {
        &mut self.db
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[tokio::test]
//     async fn create_db() {
//         tracing_setup::span!("test");
//         let _ctx = PostgresTestContext::new("create_db").await;
//
//         // Sleep for 1 minute
//         tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
//     }
// }
