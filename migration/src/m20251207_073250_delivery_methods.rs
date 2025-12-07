use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum DeliveryMethods {
    Table,
    Name,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(m, "delivery_methods",
            &[
            
            ("id", ColType::PkAuto),
            
            ("name", ColType::StringUniq),
            ("active", ColType::BooleanWithDefault(true)),
            ],
            &[
            ]
        ).await?;

        m
            .create_index(
                Index::create()
                    .name("idx-delivery_methods-name")
                    .table(DeliveryMethods::Table)
                    .col(DeliveryMethods::Name)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "delivery_methods").await
    }
}
