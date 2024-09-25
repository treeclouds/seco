use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::models::_entities::offerings;
use crate::models::_entities::sea_orm_active_enums::OfferingStatus;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AddNegotiationProductResponse {
    pub id: i32,
    #[schema(value_type = f64)]
    pub offer_price: Decimal,
    #[schema(value_type = String)]
    pub status: Option<OfferingStatus>,
    pub firebase_id: Option<String>,
    pub product_id: i32,
}

impl crate::views::offering::AddNegotiationProductResponse {
    #[must_use]
    pub fn new(offering: &offerings::Model) -> Self {
        Self {
            id: offering.id,
            offer_price: offering.offer_price,
            status: Option::from(offering.status.clone()),
            firebase_id: offering.firebase_id.clone(),
            product_id: offering.product_id,
        }
    }
}