use db::{Db, LibsqlDb};

#[tokio::main]
async fn main() {
    let _ = LibsqlDb::new_local("db/libsql/local.db")
        .await
        .inspect_err(|e| {
            tracing::error!(?e, "Failed to create connection");
        })
        .unwrap();
}
