use crate::test_utils::TestContext;
use crate::{Db, LibsqlDb};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub(crate) struct LibsqlTestContext {
    db: LibsqlDb,
    test_db_path: PathBuf,
}

impl TestContext<LibsqlDb> for LibsqlTestContext {
    /// Creates a new test context. Performs the following steps:
    /// 1. Subscribes to tracing_subscriber
    /// 2. Copies the actual local db to a test db
    #[tracing::instrument]
    async fn new(function_id: &str) -> Self {
        let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = project_root
            .parent()
            .expect("Failed to get workspace root")
            .parent()
            .expect("Failed to get workspace root");
        let test_db_path = project_root
            .join("tests/resources/db")
            .join(function_id)
            .with_extension("db");
        create_parent_dirs(&test_db_path);
        let migrations_path = workspace_root.join("migrations");

        tracing::trace!(
            ?project_root,
            ?workspace_root,
            ?test_db_path,
            ?migrations_path,
            "Creating test context"
        );

        let db_related_files = db_related_files(test_db_path.clone());
        delete_files_if_exist(db_related_files);

        // Create a connection to the test db
        let db = LibsqlDb::new_local(&test_db_path)
            .await
            .inspect_err(|e| tracing::error!(?e, "Failed to create connection"))
            .unwrap();

        // Perform migration scripts to initialize the test db
        let test_db_url = format!("sqlite://{}", test_db_path.display());
        geni::migrate_database(
            test_db_url.to_string(),
            None,
            "migrations".to_string(),
            migrations_path.display().to_string(),
            "schema.sql".to_string(),
            Some(30),
            false,
        )
        .await
        .inspect_err(|e| {
            tracing::error!(?e, "Failed to migrate database");
        })
        .unwrap();
        tracing::trace!(?test_db_path, "Database migrated");

        Self { db, test_db_path }
    }

    fn db(&self) -> &LibsqlDb {
        &self.db
    }
}

impl Drop for LibsqlTestContext {
    fn drop(&mut self) {
        let db_related_files = db_related_files(self.test_db_path.clone());
        delete_files_if_exist(db_related_files);
    }
}

fn create_parent_dirs(path: &Path) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .inspect_err(|e| {
                tracing::error!(?e, "Failed to create parent directories");
            })
            .unwrap();
    }
}

/// Returns a list of files related to a `.db` file, which are `.db-shm` and `.db-wal`.
/// Also returns the argument path itself.
#[tracing::instrument(skip(db_path))]
fn db_related_files(db_path: PathBuf) -> Vec<PathBuf> {
    if db_path.extension() != Some(OsStr::new("db")) {
        panic!("function `db_related_files` only accepts paths with the `.db` extension");
    }

    let db_related_extensions = ["db-shm", "db-wal"];

    let db_related_files: Vec<PathBuf> = db_related_extensions
        .iter()
        .map(|ext| {
            let mut db_related_path = db_path.to_path_buf();
            db_related_path.set_extension(ext);
            db_related_path
        })
        .collect();

    let mut db_related_files = db_related_files;
    db_related_files.push(db_path.to_path_buf());

    db_related_files
}

#[tracing::instrument(skip(files))]
fn delete_files_if_exist(files: Vec<PathBuf>) {
    files.into_iter().for_each(|path| {
        if path.exists() {
            std::fs::remove_file(&path)
                .inspect_err(|e| {
                    tracing::error!(?e, "Failed to delete file");
                })
                .unwrap();
            tracing::trace!(?path, "Deleted file");
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::smes::CompanyDb;
    use crate::test_utils::libsql::LibsqlTestContext;
    use crate::test_utils::*;

    #[tokio::test]
    async fn db_source_migration_should_create_empty_companies_table() {
        let function_id = utils::function_id!();
        let ctx = LibsqlTestContext::new(&function_id).await;
        let companies = ctx.db.get_companies().await.unwrap();
        assert_eq!(companies.len(), 0);
    }

    #[tokio::test]
    async fn db_source_local_should_create_working_connection() {
        if std::env::var("CI").is_ok() {
            return;
        }

        let function_id = utils::function_id!();
        let ctx = LibsqlTestContext::new(&function_id).await;
        ctx.db
            .health_check()
            .await
            .inspect_err(|e| {
                tracing::error!(?e, "Failed to health check");
            })
            .unwrap();
    }
}
