use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        const UP: &str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/sql/m20220101_000001_create_table_up.sql"
        ));

        let db = manager.get_connection();
        db.execute_unprepared(UP).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        const DOWN: &str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/sql/m20220101_000001_create_table_down.sql"
        ));

        let db = manager.get_connection();
        db.execute_unprepared(DOWN).await?;

        Ok(())
    }
}
