use sea_orm_migration::{prelude::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        let foreign_key_category = TableForeignKey::new()
            .name("fk-products-categories")
            .from_tbl(Products::Table)
            .from_col(Products::CategoriesId)
            .to_tbl(Categories::Table)
            .to_col(Categories::Id)
            .on_delete(ForeignKeyAction::Restrict)
            .on_update(ForeignKeyAction::Cascade)
            .to_owned();

        manager
            .alter_table(
                Table::alter()
                    .table(Products::Table)
                    .rename_column(Products::Category, Products::CategoriesId)
                    .to_owned()
            )
            .await?;

        let db = manager.get_connection();
        db.execute_unprepared(
            "ALTER TABLE products ALTER COLUMN category_id TYPE integer USING (REGEXP_REPLACE(COALESCE(category_id, NULL), '[^0-9]*' , NULL)::integer), ALTER COLUMN category_id DROP NOT NULL;"
        ).await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Products::Table)
                    .add_foreign_key(&foreign_key_category)
                    .to_owned()
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx-products-category")
                    .table(Products::Table)
                    .to_owned()
            )
            .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Products {
    Table,
    Category,
    #[sea_orm(iden = "category_id")]
    CategoriesId,

}

#[derive(DeriveIden)]
enum Categories {
    Table,
    Id,

}

