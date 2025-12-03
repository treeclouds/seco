use loco_rs::prelude::*;
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

pub fn home(v: impl ViewRenderer) -> Result<impl IntoResponse> {
    format::render().view(&v, "home/login.html", data!({"test": "test"}))
}

pub fn dashboard(v: impl ViewRenderer) -> Result<impl IntoResponse> {
    format::render().view(&v, "dashboard/index.html", data!({"test": "test"}))
}