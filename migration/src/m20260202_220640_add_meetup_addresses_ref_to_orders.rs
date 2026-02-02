use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let foreign_key_meetup_address = TableForeignKey::new()
            .name("fk-meetup_addresses-meetup_address_id-to-orders")
            .from_tbl(Orders::Table)
            .from_col(Orders::MeetupAddressId)
            .to_tbl(MeetupAddresses::Table)
            .to_col(MeetupAddresses::Id)
            .on_delete(ForeignKeyAction::SetNull)
            .on_update(ForeignKeyAction::Cascade)
            .to_owned();
        m
            .alter_table(
                Table::alter()
                    .table(Orders::Table)
                    .add_column_if_not_exists(ColumnDef::new("meetup_address_id").integer().null())
                    .add_column_if_not_exists(ColumnDef::new("meetup_address_detail").json().null())
                    .add_foreign_key(&foreign_key_meetup_address)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m
            .alter_table(
                Table::alter()
                    .table(Orders::Table)
                    .drop_foreign_key("fk-meetup_addresses-meetup_address_id-to-orders")
                    .to_owned(),
            )
            .await?;
        m
            .alter_table(
                Table::alter()
                    .table(Orders::Table)
                    .drop_column("meetup_address_id")
                    .to_owned(),
            )
            .await?;
        m
            .alter_table(
                Table::alter()
                    .table(Orders::Table)
                    .drop_column("meetup_address_detail")
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Orders {
    Table,
    MeetupAddressId
}

#[derive(DeriveIden)]
enum MeetupAddresses {
    Table,
    Id
}
