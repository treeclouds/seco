use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(Alias::new("seller_verifications"))
                .add_column(
                    ColumnDef::new(Alias::new("ktp_text"))
                        .string()
                        .to_owned(),
                )
                .to_owned(),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(Alias::new("seller_verifications"))
                .drop_column(Alias::new("ktp_text"))
                .to_owned(),
        )
        .await
    }
}
