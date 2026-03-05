use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::models::seller_verifications::{Model, VerificationStatus};

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SellerVerificationResponse {
    pub id: i32,
    pub user_id: i32,
    pub status: VerificationStatus,
    pub reject_reason: Option<String>,
}

impl SellerVerificationResponse {
    pub fn new(model: &Model) -> Self {
        Self {
            id: model.id,
            user_id: model.user_id,
            status: model.status,
            reject_reason: model.reject_reason.clone(),
        }
    }
}
