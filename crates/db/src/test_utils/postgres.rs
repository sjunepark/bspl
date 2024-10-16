use crate::test_utils::TestContext;
use crate::{Db, PostgresDb};
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

        let db = PostgresDb::new(&connection_string).await;

        Self { db, _node: node }
    }

    fn db(&self) -> &PostgresDb {
        &self.db
    }
}
