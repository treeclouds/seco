use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        add_column(m, "users", "magic_link_token", ColType::StringNull).await?;
        add_column(m, "users", "magic_link_expiration", ColType::TimestampWithTimeZoneNull).await?;
        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        remove_column(m, "users", "magic_link_token").await?;
        remove_column(m, "users", "magic_link_expiration").await?;
        Ok(())
    }
}
