use axum::response::{Html, IntoResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct BaseResponse {
    pub status: String,
    pub message: String,
}

impl BaseResponse {
    #[must_use]
    pub fn new(status: &str, message: &str) -> Self {
        Self {
            status: status.to_string(),
            message: message.to_string(),
        }
    }
}

/// Serves the static login page (embedded at compile time).
#[must_use]
pub fn home() -> impl IntoResponse {
    Html(include_str!("../../frontend/login.html"))
}

/// Serves the static admin dashboard (embedded at compile time).
#[must_use]
pub fn dashboard() -> impl IntoResponse {
    Html(include_str!("../../frontend/dashboard.html"))
}
