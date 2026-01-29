use loco_rs::prelude::Decimal;
use serde::{Deserialize, Serialize};
use sea_orm::{FromQueryResult, JsonValue};
use utoipa::ToSchema;
use crate::models::_entities::order_items::{Model as OrderItemsModel};


#[derive(Clone, Debug, Deserialize, Serialize, ToSchema)]
pub struct OrderItemResponse {
    pub id: i32,
    pub product_name: Option<String>,
    pub product_price: Decimal,
    pub product_sku: Option<String>,
    pub qty: i32,
}

impl OrderItemResponse {
    pub fn from_model(m: OrderItemsModel) -> Self {
        Self {
            id: m.id,
            product_name: m.product_name,
            product_price: m.product_price,
            product_sku: m.product_sku,
            qty: m.qty,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, ToSchema)]
pub struct OrderResponse {
    pub order_number: String,
    pub final_price: Decimal,
    pub delivery_address_detail: Option<serde_json::Value>,
    pub payment_method_detail: Option<serde_json::Value>,
    pub status: String,
    pub order_items: Vec<OrderItemResponse>,
}

#[derive(Clone, Debug, FromQueryResult, Deserialize, Serialize, ToSchema)]
pub struct OrderDetailResponse {
    pub order_number: String,
    pub final_price: Decimal,
    pub delivery_address_detail: Option<JsonValue>,
    pub payment_method_detail: Option<JsonValue>,
    pub status: String,
    pub order_items: Option<JsonValue>,
}