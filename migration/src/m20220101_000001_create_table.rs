use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            "
                CREATE TABLE subscriptions(
                    id uuid NOT NULL,
                    PRIMARY KEY (id),
                    email TEXT NOT NULL UNIQUE,
                    name TEXT NOT NULL,
                    subscribed_at timestamptz NOT NULL
                )
            "
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection()
            .execute_unprepared("DROP TABLE subscriptions")
            .await?;
        Ok(())
    }
}
