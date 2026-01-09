use sea_orm::{FromQueryResult, JsonValue};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, FromQueryResult, Deserialize, Serialize, ToSchema)]
pub struct DeliveryMethodsAndServicesResponse {
    id: i32,
    name: String,
    active: bool,
    #[schema(value_type = String, format = Binary)]
    services: Option<JsonValue>,
    total_services: i64
}