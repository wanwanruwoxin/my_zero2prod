use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            "
                CREATE TABLE subscription_tokens (
                    subscription_token TEXT NOT NULL,
                    subscriber_id UUID NOT NULL 
                    REFERENCES subscriptions(id) ,
                    PRIMARY KEY (subscription_token)
                );
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
                DROP TABLE subscription_tokens;
            ",
        )
        .await?;
        Ok(())
    }
}

