use loco_rs::prelude::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, Deserialize, Serialize, ToSchema)]
pub struct OrderResponse {
    pub order_number: String,
    pub final_price: Decimal,
    pub delivery_address_detail: Option<serde_json::Value>,
    pub payment_method_detail: Option<serde_json::Value>,
    pub status: String,
    pub order_items: Vec<serde_json::Value>,
}