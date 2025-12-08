use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Orders {
    Table,
    OrderNumber,
    Status,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(m, "orders",
            &[
            
            ("id", ColType::PkAuto),
            
            ("order_number", ColType::StringUniq),
            ("final_price", ColType::MoneyWithDefault(0.into())),
            ("delivery_address_detail", ColType::TextNull),
            ("payment_method_detail", ColType::JsonNull),
            ("status", ColType::EnumWithDefault(
                "order_status_enum".to_string(),
                vec!["AwaitingPayment".to_string(), "AwaitingFulfillment".to_string(),
                     "AwaitingShipment".to_string(), "Shipped".to_string(),
                     "Completed".to_string(), "Returned".to_string(), "Cancelled".to_string()],
                "AwaitingPayment".to_string()
            )),
            ],
            &[
            ("user", "buyer_id"),
            ("delivery_address", ""),
            ("payment_method", ""),
            ]
        ).await?;
        m.create_index(
            Index::create()
                .name("idx-orders-order_number")
                .table(Orders::Table)
                .col(Orders::OrderNumber)
                .to_owned(),
        ).await?;
        m.create_index(
            Index::create()
                .name("idx-orders-status")
                .table(Orders::Table)
                .col(Orders::Status)
                .to_owned(),
        ).await?;
        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "orders").await
    }
}
