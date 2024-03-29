use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = table_auto(ProductImages::Table)
            .col(pk_auto(ProductImages::Id))
            .col(integer(ProductImages::ProductsId))
                .foreign_key(
                    ForeignKey::create()
                    .name("fk-product_images-products")
                    .from(ProductImages::Table, ProductImages::ProductsId)
                    .to(Products::Table, Products::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade),
                )
            .col(binary_null(ProductImages::Image))
            .to_owned();
        manager.create_table(table).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ProductImages::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ProductImages {
    Table,
    Id,
    ProductsId,
    Image,
    
}


#[derive(DeriveIden)]
enum Products {
    Table,
    Id,
}
