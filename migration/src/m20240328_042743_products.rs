use sea_orm_migration::{prelude::*, schema::*};
use sea_orm_migration::prelude::sea_query::extension::postgres::Type;
use sea_orm::{EnumIter, DeriveActiveEnum};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("condition"))
                    .values([Alias::new("BrandNew"), Alias::new("MintCondition"), Alias::new("LikeNew"), Alias::new("Excellent"), Alias::new("Good"), Alias::new("Fair")])
                    .to_owned(),
            )
            .await?;

        let table = table_auto(Products::Table)
            .col(pk_auto(Products::Id))
            .col(integer(Products::UsersId))
            .foreign_key(
                ForeignKey::create()
                .name("fk-products-users")
                .from(Products::Table, Products::UsersId)
                .to(Users::Table, Users::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade),
            )
            .col(string(Products::Category))
            .col(string(Products::Title))
            .col(text(Products::Description))
            .col(decimal(Products::Price))
            .col(float(Products::DimensionWidth))
            .col(float(Products::DimensionHeight))
            .col(float(Products::DimensionLength))
            .col(float(Products::DimensionWeight))
            .col(string_len(Products::Brand, 100))
            .col(string_len(Products::Material, 100))
            .col(integer(Products::Stock))
            .col(string_len(Products::Sku, 100))
            .col(json_null(Products::Tags))
            .col(
                ColumnDef::new(Products::Condition)
                    .enumeration(Alias::new("condition"), [Alias::new("BrandNew"), Alias::new("MintCondition"), Alias::new("LikeNew"), Alias::new("Excellent"), Alias::new("Good"), Alias::new("Fair")]),
            )
            .to_owned();
        manager.create_table(table).await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-products-category")
                    .table(Products::Table)
                    .col(Products::Category)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-products-title")
                    .table(Products::Table)
                    .col(Products::Title)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-products-condition")
                    .table(Products::Table)
                    .col(Products::Condition)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Products::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Products {
    Table,
    Id,
    Category,
    Title,
    Description,
    Condition,
    Price,
    DimensionWidth,
    DimensionHeight,
    DimensionLength,
    DimensionWeight,
    Brand,
    Material,
    Stock,
    Sku,
    Tags,
    #[sea_orm(iden = "seller_id")]
    UsersId,

}

#[derive(EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "condition")]
pub enum Condition {
    #[sea_orm(string_value = "BrandNew")]
    BrandNew,
    #[sea_orm(string_value = "MintCondition")]
    MintCondition,
    #[sea_orm(string_value = "LikeNew")]
    LikeNew,
    #[sea_orm(string_value = "Excellent")]
    Excellent,
    #[sea_orm(string_value = "Good")]
    Good,
    #[sea_orm(string_value = "Fair")]
    Fair,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    
}


