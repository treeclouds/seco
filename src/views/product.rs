use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sea_orm::prelude::Decimal;
use sea_orm::{FromQueryResult, JsonValue};
use sea_orm::entity::prelude::*;
use uuid::Uuid;
use utoipa::ToSchema;

use crate::models::_entities::sea_orm_active_enums::Condition;
use crate::models::_entities::{product_images, products};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ProductResponse {
    pub id: i32,
    pub category: String,
    pub title: String,
    pub description: String,
    #[schema(value_type = f64)]
    pub price: Decimal,
    pub dimension_width: f32,
    pub dimension_height: f32,
    pub dimension_length: f32,
    pub dimension_weight: f32,
    pub brand: String,
    pub material: String,
    pub stock: i32,
    pub sku: String,
    #[schema(value_type = String, format = Binary)]
    pub tags: Option<JsonValue>,
    #[schema(value_type = String)]
    pub condition: Option<Condition>,
    #[schema(value_type = String)]
    pub created_at: NaiveDateTime,
    #[schema(value_type = String, format = Binary)]
    pub images: Option<Vec<ProductImageResponse>>,
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
            tags: product.tags.to_owned(),
            condition: product.condition.to_owned(),
            created_at: product.created_at.to_owned(),
            images: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProductImageResponse {
    pub id: i32,
    pub image: String,
}

impl ProductImageResponse {
    #[must_use]
    pub fn new(product_image: &product_images::Model) -> Self {
        Self {
            id: product_image.id,
            image: product_image.image.to_string(),
        }
    }
}

#[derive(Debug, FromQueryResult, Deserialize, Serialize, ToSchema)]
pub struct ProductsResponse {
    id: i32,
    category: String,
    title: String,
    description: String,
    price: Decimal,
    dimension_width: f32,
    dimension_height: f32,
    dimension_length: f32,
    dimension_weight: f32,
    brand: String,
    material: String,
    stock: i32,
    sku: String,
    tags: Option<JsonValue>,
    condition: Option<String>,
    created_at: DateTime,
    images: Option<JsonValue>,
    seller_pid: Uuid,
    seller_first_name: String,
    seller_last_name: String,
    seller_joined_date: DateTime,
}
