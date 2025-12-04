use loco_rs::schema::*;
use sea_orm_migration::prelude::*;
use crate::sea_orm::{DbBackend, DeriveActiveEnum, EnumIter, Schema};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let schema = Schema::new(DbBackend::Postgres);

        m.create_type(
            schema.create_enum_from_active_enum::<ProductStatus>(),
        ).await?;

        add_column(m, "products", "status", ColType::EnumWithDefault("product_status".to_string(), vec!["Draft".to_string(), "Active".to_string(), "Sold".to_string()], "Active".to_string())).await?;
        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        remove_column(m, "products", "status").await?;
        Ok(())
    }
}

#[derive(EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "product_status")]
pub enum ProductStatus {
    #[sea_orm(string_value = "Draft")]
    Draft,
    #[sea_orm(string_value = "Active")]
    Active,
    #[sea_orm(string_value = "Sold")]
    Sold,
}
