//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "product_images")]
pub struct Model {
    pub created_at: DateTime,
    pub updated_at: DateTime,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub products_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::products::Entity",
        from = "Column::ProductsId",
        to = "super::products::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Products,
}

impl Related<super::products::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Products.def()
    }
}
