use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sea_orm::prelude::Decimal;
use sea_orm::{FromQueryResult, JsonValue};
use sea_orm::entity::prelude::*;
use utoipa::ToSchema;

use crate::models::_entities::sea_orm_active_enums::Condition;
use crate::models::_entities::{product_images, products};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ProductResponse {
    pub id: i32,
    pub category_id: Option<i32>,
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
            category_id: product.category_id,
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
    category_id: Option<i32>,
    title: String,
    description: String,
    #[schema(value_type = f64)]
    price: Decimal,
    dimension_width: f32,
    dimension_height: f32,
    dimension_length: f32,
    dimension_weight: f32,
    brand: String,
    material: String,
    stock: i32,
    sku: String,
    #[schema(value_type = String, format = Binary)]
    tags: Option<JsonValue>,
    #[schema(value_type = String)]
    condition: Option<String>,
    #[schema(value_type = String)]
    created_at: DateTime,
    #[schema(value_type = String, format = Binary)]
    images: Option<JsonValue>,
    #[schema(value_type = String, format = Binary)]
    seller: Option<JsonValue>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ProductDealResponse {
    pub id: i32,
    pub category_id: Option<i32>,
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
    pub condition: Option<String>,
    #[schema(value_type = String)]
    pub created_at: DateTime,
    #[schema(value_type = String, format = Binary)]
    pub images: Option<JsonValue>,
    #[schema(value_type = String, format = Binary)]
    pub seller: Option<JsonValue>,
    #[schema(value_type = f64)]
    pub price_deal: Option<Decimal>,
}

impl ProductDealResponse {
    #[must_use]
    pub fn new(product: ProductsResponse, price_deal: Option<Decimal>) -> Self {
        Self {
            id: product.id,
            category_id: product.category_id,
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
            images: product.images,
            seller: product.seller,
            price_deal: Option::from(price_deal),
        }
    }
}
