use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(m, "payment_methods",
            &[
            
            ("id", ColType::PkAuto),
            
            ("name", ColType::EnumWithDefault(
                "payment_method_name_enum".to_string(),
                vec!["COD".to_string(), "QRIS".to_string(), "VirtualAccount".to_string(), "BankTransfer".to_string()],
                "COD".to_string()
            )),
            ("description", ColType::TextNull),
            ("expiry_time", ColType::IntegerWithDefault(60)),
            ("payment_gateway", ColType::EnumNull(
                "payment_method_gateway_enum".to_string(),
                vec!["Xendit".to_string(), "Tripay".to_string(), "Midtrans".to_string()]
            )),
            ("active", ColType::BooleanWithDefault(true)),
            ],
            &[
            ]
        ).await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "payment_methods").await
    }
}
