use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::models::_entities::{
    seller_verifications,
    sea_orm_active_enums::VerificationStatus
};

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SellerVerificationResponse {
    pub id: i32,
    pub user_id: i32,
    pub status: VerificationStatus,
    pub reject_reason: Option<String>,
    pub ktp_text: Option<String>,
}

impl SellerVerificationResponse {
    pub fn new(model: &seller_verifications::Model) -> Self {
        Self {
            id: model.id,
            user_id: model.user_id,
            status: model.status,
            reject_reason: Option::from(model.reject_reason.clone()),
            ktp_text: Option::from(model.ktp_text.clone()),
        }
    }
}
