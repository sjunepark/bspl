use crate::test_utils::TestContext;
use crate::PostgresDb;
use diesel::{Connection, PgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
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
            .with_tag("16")
            .start()
            .await
            .expect("Failed to start container");

        let connection_string = format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            node.get_host_port_ipv4(5432)
                .await
                .expect("Failed to get port for test db connection")
        );
        tracing::trace!(%connection_string, "Connection string");

        // Run migrations via diesel
        tracing::trace!("Running migrations");
        let mut conn = PgConnection::establish(&connection_string)
            .expect("Failed to establish connection to test db");
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../../migrations");
        let migration_versions = conn
            .run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
        tracing::debug!(?migration_versions, "Migrations ran successfully");

        let db = PostgresDb { conn };

        Self { db, _node: node }
    }

    fn db(&mut self) -> &mut PostgresDb {
        &mut self.db
    }
}
