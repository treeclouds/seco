use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.create_type(
            sea_query::extension::postgres::Type::create()
                .as_enum(Alias::new("verification_status"))
                .values([
                    Alias::new("Pending"),
                    Alias::new("Approved"),
                    Alias::new("Rejected"),
                ])
                .to_owned(),
        )
        .await?;

        create_table(
            m,
            "seller_verifications",
            &[
                ("id", ColType::PkAuto),

                ("face_photo", ColType::Text),
                ("ktp_photo", ColType::Text),
                ("face_ktp_photo", ColType::Text),
                ("domicile_photo", ColType::Text),
                ("status", ColType::String),
                ("reject_reason", ColType::Text),
            ],
            &[("user", "")],
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "seller_verifications").await?;
        m.drop_type(
            sea_query::extension::postgres::Type::drop()
                .name(Alias::new("verification_status"))
                .to_owned(),
        )
        .await
    }
}
