use loco_rs::schema::table_auto_tz;
use sea_orm_migration::{prelude::*, schema::*};
use sea_orm::{Schema, DbBackend};
use crate::sea_orm::{DeriveActiveEnum, EnumIter};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        let schema = Schema::new(DbBackend::Postgres);

        manager
            .create_type(
                schema.create_enum_from_active_enum::<OfferingStatus>(),
            )
            .await?;

        manager
            .create_type(
                schema.create_enum_from_active_enum::<ActionType>(),
            )
            .await?;

        manager
            .create_table(
                table_auto_tz(Offerings::Table)
                    .col(pk_auto(Offerings::Id))
                    .col(integer(Offerings::ProductId))
                    .col(integer(Offerings::UserId))
                    .col(string(Offerings::ProductName))
                    .col(string(Offerings::ProductCondition))
                    .col(decimal_len(Offerings::ProductPrice, 16, 2))
                    .col(decimal_len(Offerings::OfferPrice, 16, 2))
                    .col(
                        ColumnDef::new(Offerings::Status)
                            .custom(Alias::new("offering_status")) // Use the enum type name
                            .null()
                            .default("InProgress")
                            .to_owned(),
                    )
                    .col(
                        ColumnDef::new(Offerings::ActionType)
                            .custom(Alias::new("action_type")) // Use the enum type name
                            .null()
                            .to_owned(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-offerings-products")
                            .from(Offerings::Table, Offerings::ProductId)
                            .to(Products::Table, Products::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-offerings-users")
                            .from(Offerings::Table, Offerings::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Offerings::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Offerings {
    Table,
    Id,
    ProductId,
    UserId,
    ProductName,
    ProductCondition,
    ProductPrice,
    OfferPrice,
    Status,
    ActionType,
    
}


#[derive(DeriveIden)]
enum Products {
    Table,
    Id,
}
#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}

#[derive(EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "offering_status")]
pub enum OfferingStatus {
    #[sea_orm(string_value = "InProgress")]
    InProgress,
    #[sea_orm(string_value = "Accepted")]
    Accepted,
    #[sea_orm(string_value = "Declined")]
    Declined,
}

#[derive(EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "action_type")]
pub enum ActionType {
    #[sea_orm(string_value = "CounterOffer")]
    CounterOffer,
    #[sea_orm(string_value = "Accept")]
    Accept,
    #[sea_orm(string_value = "Decline")]
    Decline,
}
