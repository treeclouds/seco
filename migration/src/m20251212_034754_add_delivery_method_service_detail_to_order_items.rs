use loco_rs::schema::{add_column, remove_column, ColType};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        add_column(m, "order_items", "delivery_method_service_detail", ColType::JsonNull).await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        remove_column(m, "order_items", "delivery_method_service_detail").await
    }
}

