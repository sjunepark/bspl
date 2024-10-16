//! # Benchmarks for libsql inserts
//!
//! ## `bench_glob_check`
//! This function benchmarks the performance effect of using `GLOB` checks in SQLite.
//! It benchmarks th following cases:
//! 1. Without `GLOB` check: Inserts data without using `GLOB` checks.
//! 2. With `GLOB` check: Inserts data using `GLOB` checks.
//!
//! ### Results
//! The results show that using `GLOB` checks can have a negligible effect on performance.
//!
//!
//! ## `bench_libsql_insert`
//!
//! This function contains benchmarks for libsql inserts, for the following cases:
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
use db::{Db, DbError, LibsqlDb};
use libsql::Connection;
use rand::Rng;
use tokio::time::Instant;

criterion_group!(benches, bench_libsql_insert, bench_glob_check);
criterion_main!(benches);

fn bench_glob_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("libsql_glob_check");
    let size: usize = 128;

    for size in [size, size * 4, size * 16].iter() {
        group.bench_with_input(
            BenchmarkId::new("without_glob_check", size),
            &size,
            |b, &&size| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter_custom(|iters| async move {
                        let db = BenchDb::new().await;
                        db.create_names_table_without_glob_check()
                            .await
                            .inspect_err(|e| tracing::error!(?e, "Failed to create names table"))
                            .unwrap();
                        let names = Name::create_test_data(size);
                        let start = Instant::now();
                        for _ in 0..iters {
                            db.insert_names(&names).await.unwrap();
                        }
                        start.elapsed()
                    });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("with_glob_check", size),
            &size,
            |b, &&size| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter_custom(|iters| async move {
                        let db = BenchDb::new().await;
                        db.create_names_table_with_glob_check()
                            .await
                            .inspect_err(|e| tracing::error!(?e, "Failed to create names table"))
                            .unwrap();
                        let names = Name::create_test_data(size);
                        let start = Instant::now();
                        for _ in 0..iters {
                            db.insert_names(&names).await.unwrap();
                        }
                        start.elapsed()
                    });
            },
        );
    }
}

fn bench_libsql_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("libsql_insert");
    let size: usize = 128;

    for size in [size, size * 4, size * 16].iter() {
        group.bench_with_input(
            BenchmarkId::new("naive_insert", size),
            &size,
            |b, &&size| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter_custom(|iters| async move {
                        let db = BenchDb::new().await;
                        db.create_users_table()
                            .await
                            .inspect_err(|e| tracing::error!(?e, "Failed to create users table"))
                            .unwrap();
                        let users = User::create_test_data(size);
                        let start = Instant::now();
                        for _ in 0..iters {
                            db.naive_insert(&users).await.unwrap();
                        }
                        start.elapsed()
                    });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("prepared_insert", size),
            &size,
            |b, &&size| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter_custom(|iter| async move {
                        let db = BenchDb::new().await;
                        db.create_users_table()
                            .await
                            .inspect_err(|e| tracing::error!(?e, "Failed to create users table"))
                            .unwrap();
                        let users = User::create_test_data(size);
                        let start = Instant::now();
                        for _ in 0..iter {
                            db.prepared_insert(&users).await.unwrap();
                        }
                        start.elapsed()
                    });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("transaction_insert", size),
            &size,
            |b, &&size| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter_custom(|iter| async move {
                        let db = BenchDb::new().await;
                        db.create_users_table()
                            .await
                            .inspect_err(|e| tracing::error!(?e, "Failed to create users table"))
                            .unwrap();
                        let users = User::create_test_data(size);
                        let start = Instant::now();
                        for _ in 0..iter {
                            db.transaction_insert(&users).await.unwrap();
                        }
                        start.elapsed()
                    });
            },
        );
    }
}

#[derive(Debug)]
struct Name(String);

impl Name {
    fn create_test_data(size: usize) -> Vec<Self> {
        let mut rng = rand::thread_rng();
        (0..size)
            .map(|_| Name(rng.gen_range(0..1000).to_string()))
            .collect()
    }
}

#[derive(Clone)]
struct User {
    name: String,
    age: i64,
    vision: f64,
    avatar: Vec<u8>,
}

impl User {
    fn create_test_data(size: usize) -> Vec<Self> {
        let mut rng = rand::thread_rng();
        (0..size)
            .map(|_| User {
                name: rng.gen_range(0..100).to_string(),
                age: rng.gen_range(0..100),
                vision: rng.gen_range(0.0..1.0),
                avatar: vec![0; 1024],
            })
            .collect()
    }
}

struct BenchDb {
    connection: Connection,
}

impl BenchDb {
    async fn new() -> Self {
        let db = LibsqlDb::new(":memory:").await;

        Self {
            connection: db.connection,
        }
    }

    async fn create_users_table(&self) -> Result<(), DbError> {
        self.connection
            .execute(
                "CREATE TABLE users (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                age INTEGER NOT NULL,
                vision REAL NOT NULL,
                avatar BLOB NOT NULL
         )",
                (),
            )
            .await?;
        Ok(())
    }

    async fn create_names_table_without_glob_check(&self) -> Result<(), DbError> {
        self.connection
            .execute(
                "CREATE TABLE names (
                name TEXT NOT NULL
            )",
                (),
            )
            .await?;
        Ok(())
    }

    async fn create_names_table_with_glob_check(&self) -> Result<(), DbError> {
        self.connection
            .execute(
                "CREATE TABLE names
(
    name TEXT NOT NULL CHECK ( name GLOB '[0-9]*' )
)",
                (),
            )
            .await?;
        Ok(())
    }

    async fn naive_insert(&self, data: &[User]) -> Result<(), DbError> {
        for user in data {
            self.connection
                .execute(
                    "INSERT INTO users (name, age, vision, avatar) VALUES (:name, :age, :vision, :avatar)",
                    libsql::named_params! {":name": &user.name.as_str(), ":age": user.age, ":vision": user.vision, ":avatar": &user.avatar.as_slice()},
                )
                .await?;
        }
        Ok(())
    }

    async fn prepared_insert(&self, data: &[User]) -> Result<(), DbError> {
        let mut stmt = self
            .connection
            .prepare("INSERT INTO users (name, age, vision, avatar) VALUES (:name, :age, :vision, :avatar)")
            .await?;
        for user in data {
            stmt.execute(libsql::named_params! {":name": &user.name.as_str(), ":age": user.age, ":vision": user.vision, ":avatar": &user.avatar.as_slice()}).await?;
            stmt.reset();
        }
        Ok(())
    }

    async fn transaction_insert(&self, data: &[User]) -> Result<(), DbError> {
        let tx = self.connection.transaction().await?;
        let mut stmt = tx
            .prepare("INSERT INTO users (name, age, vision, avatar) VALUES (:name, :age, :vision, :avatar)")
            .await?;
        for user in data {
            stmt.execute(libsql::named_params! {":name": &user.name.as_str(), ":age": user.age, ":vision": user.vision, ":avatar": &user.avatar.as_slice()}).await?;
            stmt.reset();
        }
        tx.commit().await?;
        Ok(())
    }

    async fn insert_names(&self, data: &[Name]) -> Result<(), DbError> {
        let mut stmt = self
            .connection
            .prepare("INSERT INTO names (name) VALUES (:name)")
            .await?;
        for name in data {
            stmt.execute(libsql::named_params! {":name": &name.0.as_str()})
                .await?;
            stmt.reset();
        }
        Ok(())
    }
}
