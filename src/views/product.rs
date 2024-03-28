use serde::{Deserialize, Serialize};
use sea_orm::prelude::Decimal;

use crate::models::_entities::products;

#[derive(Debug, Deserialize, Serialize)]
pub struct ProductResponse {
    pub id: i32,
    pub category: String,
    pub title: String,
    pub description: String,
    pub price: Decimal,
    pub dimension_width: f32,
    pub dimension_height: f32,
    pub dimension_length: f32,
    pub dimension_weight: f32,
    pub brand: String,
    pub material: String,
    pub stock: i32,
    pub sku: String,
}

impl ProductResponse {
    #[must_use]
    pub fn new(product: &products::Model) -> Self {
        Self {
            id: product.id,
            category: product.category.to_string(),
            title: product.title.to_string(),
            description: product.description.to_string(),
            price: product.price,
            dimension_width: product.dimension_width,
            dimension_height: product.dimension_height,
            dimension_length: product.dimension_length,
            dimension_weight: product.dimension_weight,
            brand: product.brand.to_string(),
            material: product.material.to_string(),
            stock: product.stock,
            sku: product.sku.to_string(),
        }
    }
}
