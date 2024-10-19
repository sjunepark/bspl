//! # Benchmarks for postgres inserts
//!
//! ## `bench_glob_check`
//! This function benchmarks the performance effect of using `GLOB` checks in SQLite.
//! It benchmarks the following cases:
//! 1. Without `GLOB` check: Inserts data without using `GLOB` checks.
//! 2. With `GLOB` check: Inserts data using `GLOB` checks.
//!
//! ### Results
//! The results show that using `GLOB` checks can have a negligible effect on performance.
//!
//!
//! ## `bench_postgres_insert`
//!
//! This function contains benchmarks for postgres inserts, for the following cases:
//! 1. Naive insert: Inserting data without using prepared statements.
//! 2. Prepared insert: Inserting data using prepared statements.
//! 3. Transaction insert: In addition to 2., uses a transaction to insert data.
//!
//! The measurement doesn't account for cleanup,
//! meaning that the table will grow in size as inserts are performed.
//! However, the overall trend should be the same, even when the above is properly considered.
//!
//! ### Results
//! The results show that using prepared statements with transactions is the fastest way to insert data.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use db::DbError;
use diesel::{Connection, PgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use fake::{Fake, Faker};
use model::table;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, QueryBuilder};
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::{ContainerAsync, ImageExt};
use tokio::time::Instant;

criterion_group!(benches, bench_postgres_insert);
criterion_main!(benches);

fn bench_postgres_insert(c: &mut Criterion) {
    tracing_setup::span!("bench_postgres_insert");

    let mut group = c.benchmark_group("postgres_insert");
    group.sample_size(10);
    const TEST_SIZE: u64 = 10000;

    for size in [TEST_SIZE, TEST_SIZE * 4].iter() {
        group.bench_with_input(BenchmarkId::new("naive_insert", size), size, |b, size| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter_custom(|iters| async move {
                    // Can't reuse the same db since sqlx randomly throws PoolTimeout errors during tests.
                    // Setting max connection to 1 doesn't fix the issue.
                    // <https://github.com/launchbadge/sqlx/issues/2567>
                    let db = BenchDb::new().await;
                    let htmls = create_html_data(*size);
                    let mut total_time = std::time::Duration::default();
                    for _ in 0..iters {
                        let start = Instant::now();
                        db.naive_insert(&htmls).await.unwrap();
                        total_time += start.elapsed();

                        sqlx::query!("DELETE FROM smes.html")
                            .execute(&db.pool)
                            .await
                            .expect("Failed to delete all rows");
                    }
                    total_time
                });
        });
        group.bench_with_input(
            BenchmarkId::new("prepared_insert", size),
            size,
            |b, size| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter_custom(|iters| async move {
                        let db = BenchDb::new().await;
                        let htmls = create_html_data(*size);
                        let mut total_time = std::time::Duration::default();
                        for _ in 0..iters {
                            let start = Instant::now();
                            db.prepared_insert(&htmls).await.unwrap();
                            total_time += start.elapsed();

                            sqlx::query!("DELETE FROM smes.html")
                                .execute(&db.pool)
                                .await
                                .expect("Failed to delete all rows");
                        }
                        total_time
                    });
            },
        );
    }
}

#[tracing::instrument]
fn create_html_data(size: u64) -> Vec<table::Html> {
    if size > 9000000 {
        panic!("size must be less than 9000000");
    }

    let ids = (0..size).map(|i| 1000000 + i).collect::<Vec<_>>();
    tracing::trace!(?ids, "Created IDs");
    let htmls = ids
        .iter()
        .map(|id| {
            let html = Faker.fake::<table::Html>();
            table::Html {
                company_id: id
                    .to_string()
                    .try_into()
                    .expect("failed to create dummy company_id"),
                ..html
            }
        })
        .collect();
    tracing::trace!("Created HTML data");
    htmls
}

struct BenchDb {
    pool: PgPool,
    _node: ContainerAsync<Postgres>,
}

impl BenchDb {
    #[tracing::instrument]
    async fn new() -> Self {
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

        // This is to prevent sqlx from returning timeout errors.
        let max_connection = 1;

        let pool = PgPoolOptions::new()
            .max_connections(max_connection)
            .connect(&connection_string)
            .await
            .expect("Failed to create connection pool");

        // Run migrations via diesel
        let mut conn = PgConnection::establish(&connection_string)
            .expect("Failed to establish connection to test db");
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../../migrations");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
        tracing::trace!("Migrations run");

        sqlx::query!("ALTER TABLE smes.html DROP CONSTRAINT html_company_id_fkey")
            .execute(&pool)
            .await
            .expect("Failed to drop constraint");
        tracing::trace!("Constraint dropped");

        Self { pool, _node: node }
    }

    #[tracing::instrument(skip(self, htmls))]
    async fn naive_insert(&self, htmls: &[table::Html]) -> Result<(), DbError> {
        let mut insert_count = 0;

        for html in htmls {
            let rows_affected = sqlx::query!(
                "INSERT INTO smes.html(company_id, html_raw) VALUES($1, $2)",
                html.company_id.to_string(),
                html.html.to_string()
            )
            .execute(&self.pool)
            .await?
            .rows_affected();
            insert_count += rows_affected;
        }
        tracing::trace!(?insert_count, "Naive INSERT INTO users");
        Ok(())
    }

    async fn prepared_insert(&self, htmls: &[table::Html]) -> Result<(), DbError> {
        // <https://docs.rs/sqlx/latest/sqlx/struct.QueryBuilder.html#method.push_bind>
        const BIND_LIMIT: usize = 65535;
        let mut total_insert_count = 0;

        for html_chunk in htmls.chunks(BIND_LIMIT / 4) {
            let mut query_builder =
                QueryBuilder::new("INSERT INTO smes.html(company_id, html_raw) ");
            query_builder.push_values(html_chunk.iter(), |mut b, html| {
                b.push_bind(html.company_id.to_string())
                    .push_bind(html.html.to_string());
            });
            let query = query_builder.build();
            let insert_count = query.execute(&self.pool).await?.rows_affected();
            total_insert_count += insert_count;
        }

        tracing::trace!(?total_insert_count, "Prepared INSERT INTO users");
        Ok(())
    }
}
