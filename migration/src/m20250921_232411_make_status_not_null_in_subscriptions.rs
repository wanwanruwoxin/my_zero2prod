use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            "
                BEGIN;
                UPDATE subscriptions SET status = 'confirmed' WHERE status IS NULL;
                ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;
                COMMIT;
            ",
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        let db = manager.get_connection();
        db.execute_unprepared(
            "
                ALTER TABLE subscriptions ALTER COLUMN status DROP NOT NULL;
            ",
        )
        .await?;
        Ok(())
    }
}

