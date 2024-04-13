use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
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