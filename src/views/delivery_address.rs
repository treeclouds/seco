use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::models::_entities::delivery_addresses;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DeliveryAddressResponse {
    pub id: i32,
    pub name: String,
    pub phone: String,
    pub email: String,
    pub city: String,
    pub postal_code: String,
    pub address: String,
    pub notes: Option<String>,
    pub latitude: Option<String>,
    pub longitude: Option<String>,
}

impl DeliveryAddressResponse {
    #[must_use]
    pub fn new(delivery_address: &delivery_addresses::Model) -> Self {
        Self {
            id: delivery_address.id,
            name: delivery_address.name.to_string(),
            phone: delivery_address.name.to_string(),
            email: delivery_address.name.to_string(),
            city: delivery_address.name.to_string(),
            postal_code: delivery_address.name.to_string(),
            address: delivery_address.name.to_string(),
            notes: Option::from(delivery_address.notes.clone()),
            latitude: Option::from(delivery_address.latitude.clone()),
            longitude: Option::from(delivery_address.longitude.clone()),
        }
    }
}