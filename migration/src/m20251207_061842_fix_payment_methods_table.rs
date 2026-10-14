use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum PaymentMethods {
    Table,
    Name,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared(
            "ALTER TABLE payment_methods ADD CONSTRAINT payment_methods_name_key UNIQUE (name)"
        ).await?;

        m
            .create_index(
                Index::create()
                    .name("idx-payment_methods-name")
                    .table(PaymentMethods::Table)
                    .col(PaymentMethods::Name)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared(
            "ALTER TABLE payment_methods DROP CONSTRAINT payment_methods_name_key"
        ).await?;

        m
            .drop_index(
                Index::drop()
                    .name("idx-payment_methods-name")
                    .table(PaymentMethods::Table)
                    .to_owned()
            )
            .await?;
        Ok(())
    }
}

