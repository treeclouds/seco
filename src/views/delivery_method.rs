use sea_orm::{FromQueryResult, JsonValue};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, FromQueryResult, Deserialize, Serialize, ToSchema)]
pub struct DeliveryMethodsAndServicesResponse {
    name: String,
    #[schema(value_type = String, format = Binary)]
    services: Option<JsonValue>,
}