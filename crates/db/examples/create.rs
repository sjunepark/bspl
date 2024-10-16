use db::{Db, LibsqlDb};

#[tokio::main]
async fn main() {
    let _ = LibsqlDb::new("db/libsql/local.db").await;
}
