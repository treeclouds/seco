use serde::{Deserialize, Serialize};
use sea_orm::{FromQueryResult, JsonValue};
use utoipa::ToSchema;

#[derive(Debug, FromQueryResult, Deserialize, Serialize, ToSchema)]
pub struct WishlistListResponse {
    pub id: i32,
    pub product_detail: Option<JsonValue>,
}