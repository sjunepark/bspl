use crate::test_utils::TestContext;
use crate::PostgresDb;
use sqlx::migrate;
use sqlx::postgres::PgPoolOptions;
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

        migrate!("../../migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        let db = PostgresDb { pool };

        Self { db, _node: node }
    }

    fn db(&self) -> &PostgresDb {
        &self.db
    }
}
