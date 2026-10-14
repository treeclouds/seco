use sea_orm_migration::{prelude::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[allow(dead_code)]
#[derive(DeriveIden)]
enum Offerings {
    Table,
    ProductCondition,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        //
        // add column
        //
        /*
        manager
            .alter_table(
                Table::alter()
                    .table(Movies::Table)
                    .add_column_if_not_exists(integer(Movies::Rating))
                    .to_owned(),
            )
            .await
        */

        //
        // delete column
        //
        /*
        manager
            .alter_table(
                Table::alter()
                    .table(Movies::Table)
                    .drop_column(Movies::Rating)
                    .to_owned(),
            )
            .await
        */

        //
        // create index
        //
        /*
        manager
            .create_index(
                Index::create()
                    .name("idx-movies-rating")
                    .table(Movies::Table)
                    .col(Movies::Rating)
                    .to_owned(),
            )
            .await;
        */
        // manager
        //     .alter_table(
        //         Table::alter()
        //             .table(Offerings::Table)
        //             .modify_column(
        //                 ColumnDef::new(Offerings::ProductCondition)
        //                     .custom(Alias::new("condition")) // Use the enum type name
        //                     .null()
        //                     .to_owned(),
        //             )
        //             .to_owned(),
        //     )
        //     .await
        let db = manager.get_connection();
        db.execute_unprepared(
            "ALTER TABLE offerings ALTER COLUMN product_condition TYPE condition USING product_condition::condition, ALTER COLUMN product_condition DROP NOT NULL;"
        ).await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

