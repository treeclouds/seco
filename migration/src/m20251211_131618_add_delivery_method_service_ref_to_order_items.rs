use loco_rs::schema::{add_column, add_reference, remove_column, remove_reference, ColType};
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        add_reference(m, "order_items", "delivery_method_services", "delivery_method_service_id").await?;
        add_column(m, "order_items", "delivery_method_service_price", ColType::MoneyWithDefault(0.into())).await?;
        add_column(m, "order_items", "delivery_method_service_name", ColType::StringNull).await?;
        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        remove_reference(m, "order_items", "delivery_method_services", "delivery_method_service_id").await?;
        remove_column(m, "order_items", "delivery_method_service_price").await?;
        remove_column(m, "order_items", "delivery_method_service_name").await?;
        Ok(())
    }
}

