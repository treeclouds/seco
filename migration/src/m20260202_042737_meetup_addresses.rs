use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(m, "meetup_addresses",
            &[
            
            ("id", ColType::PkAuto),
            
            ("date", ColType::Date),
            ("time", ColType::Time),
            ("city", ColType::String),
            ("postal_code", ColType::String),
            ("address", ColType::Text),
            ("notes", ColType::TextNull),
            ],
            &[
            ("user", ""),
            ]
        ).await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "meetup_addresses").await
    }
}
