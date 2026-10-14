use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum DeliveryMethodServices {
    Table,
    Name,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(m, "delivery_method_services",
            &[
            
            ("id", ColType::PkAuto),
            
            ("name", ColType::StringUniq),
            ("active", ColType::BooleanWithDefault(true)),
            ],
            &[
            ("delivery_method", ""),
            ]
        ).await?;

        m
            .create_index(
                Index::create()
                    .name("idx-delivery_method_services-name")
                    .table(DeliveryMethodServices::Table)
                    .col(DeliveryMethodServices::Name)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "delivery_method_services").await
    }
}
