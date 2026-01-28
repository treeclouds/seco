use loco_rs::prelude::DateTime;
use sea_orm::prelude::Decimal;
use serde::{Deserialize, Serialize};
use sea_orm::{FromQueryResult, JsonValue};
use utoipa::ToSchema;

#[derive(Debug, FromQueryResult, Deserialize, Serialize, ToSchema)]
pub struct WishlistListResponse {
    id: i32,
    product_id: i32,
    category_id: Option<i32>,
    title: String,
    description: String,
    #[schema(value_type = f64)]
    price: Decimal,
    dimension_width: f32,
    dimension_height: f32,
    dimension_length: f32,
    dimension_weight: f32,
    brand_id: Option<i32>,
    material_id: Option<i32>,
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