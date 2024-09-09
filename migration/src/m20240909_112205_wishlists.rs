use loco_rs::schema::table_auto_tz;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                table_auto_tz(Wishlists::Table)
                    .col(pk_auto(Wishlists::Id))
                    .col(integer(Wishlists::UserId))
                    .col(integer(Wishlists::ProductId))
                    .col(boolean(Wishlists::IsDeleted).default(false))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-wishlists-users")
                            .from(Wishlists::Table, Wishlists::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-wishlists-products")
                            .from(Wishlists::Table, Wishlists::ProductId)
                            .to(Products::Table, Products::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Wishlists::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Wishlists {
    Table,
    Id,
    UserId,
    ProductId,
    IsDeleted,
    
}


#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
#[derive(DeriveIden)]
enum Products {
    Table,
    Id,
}
