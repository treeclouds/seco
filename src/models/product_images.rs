use super::_entities::product_images::{ActiveModel, Entity, Model};
use sea_orm::entity::prelude::*;

impl ActiveModelBehavior for ActiveModel {
    // extend activemodel below (keep comment for generators)
}

// implement your read-oriented logic here
impl Model {}

// implement your write-oriented logic here
impl ActiveModel {}

// implement your custom finders, selectors oriented logic here
impl Entity {}
