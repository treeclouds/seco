use loco_rs::schema::{add_column, remove_column, ColType};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        add_column(m, "delivery_method_services", "price", ColType::DecimalLenWithDefault(10, 2, 0.0)).await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        remove_column(m, "delivery_method_services", "price").await
    }
}

