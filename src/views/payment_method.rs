use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::models::_entities::payment_methods;
use crate::models::_entities::sea_orm_active_enums::{PaymentMethodGatewayEnum, PaymentMethodNameEnum};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct PaymentMethodResponse {
    pub id: i32,
    pub name: PaymentMethodNameEnum,
    pub description: Option<String>,
    pub expiry_time: i32,
    pub payment_gateway: Option<PaymentMethodGatewayEnum>,
    pub active: bool,
}

impl PaymentMethodResponse {
    #[must_use]
    pub fn new(payment_method: &payment_methods::Model) -> Self {
        Self {
            id: payment_method.id,
            name: payment_method.name.to_owned(),
            description: Option::from(payment_method.description.clone()),
            expiry_time: payment_method.expiry_time,
            payment_gateway: Option::from(payment_method.payment_gateway),
            active: payment_method.active,
        }
    }
}