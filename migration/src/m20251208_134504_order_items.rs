use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(m, "order_items",
            &[
            
            ("id", ColType::PkAuto),
            
            ("product_name", ColType::StringNull),
            ("product_price", ColType::MoneyNull),
            ("product_sku", ColType::StringNull),
            ("product_condition", ColType::StringNull),
            ("qty", ColType::IntegerWithDefault(1)),
            ],
            &[
            ("order", ""),
            ("user", "seller_id"),
            ("product", ""),
            ]
        ).await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "order_items").await
    }
}
