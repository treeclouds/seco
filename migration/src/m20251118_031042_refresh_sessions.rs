use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(m, "refresh_sessions",
            &[

            ("id", ColType::PkAuto),
            ("jti", ColType::String),
            ("token_hash", ColType::String),
            ("revoked", ColType::Boolean),
            ("expires_at", ColType::TimestampWithTimeZone),
            ("revoked_at", ColType::TimestampWithTimeZoneNull),
            ],
            &[
            ("user", ""),
            ]
        ).await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "refresh_sessions").await
    }
}
