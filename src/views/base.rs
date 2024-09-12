use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct BaseResponse {
    pub status: String,
    pub message: String,
}

impl BaseResponse {
    #[must_use]
    pub fn new(status: &String, message: &String) -> Self {
        Self {
            status: status.to_string(),
            message: message.to_string(),
        }
    }
}