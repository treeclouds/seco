use sea_orm::entity::prelude::*;

pub use super::_entities::products::{self, ActiveModel, Entity, Model};

#[async_trait::async_trait]
impl ActiveModelBehavior for super::_entities::products::ActiveModel {
    // extend activemodel below (keep comment for generators)
}

impl super::_entities::products::Model {}

impl super::_entities::products::ActiveModel {}
