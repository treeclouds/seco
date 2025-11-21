use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        remove_column(m, "products", "brand").await?;
        remove_column(m, "products", "material").await?;
        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        add_column(m, "products", "brand", ColType::String).await?;
        add_column(m, "products", "material", ColType::String).await?;
        Ok(())
    }
}
