use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Offerings {
    Table,
    FirebaseId,
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
        manager
            .alter_table(
                Table::alter()
                    .table(Offerings::Table)
                    .add_column_if_not_exists(string_null(Offerings::FirebaseId))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-offerings-firebase_id")
                    .table(Offerings::Table)
                    .col(Offerings::FirebaseId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

