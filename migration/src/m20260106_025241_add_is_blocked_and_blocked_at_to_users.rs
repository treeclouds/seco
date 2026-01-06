use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        add_column(m, "users", "is_blocked", ColType::BooleanWithDefault(false)).await?;
        add_column(m, "users", "blocked_at", ColType::TimestampWithTimeZoneNull).await?;
        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        remove_column(m, "users", "is_blocked").await?;
        remove_column(m, "users", "blocked_at").await?;
        Ok(())
    }
}
