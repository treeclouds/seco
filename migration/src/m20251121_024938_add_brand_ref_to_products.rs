use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        add_reference(m, "products", "brand", "").await?;
        let db = m.get_connection();
        db.execute_unprepared(
            "ALTER TABLE products ALTER COLUMN brand_id DROP NOT NULL;"
        ).await?;
        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        remove_reference(m, "products", "brand", "").await?;
        Ok(())
    }
}
