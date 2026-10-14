use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::_entities::users;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CurrentResponse {
    pub pid: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub location: Option<String>,
}

impl CurrentResponse {
    #[must_use]
    pub fn new(user: &users::Model) -> Self {
        Self {
            pid: user.pid.to_string(),
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
            email: user.email.clone(),
            location: Option::from(user.location.clone()),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AdminUserResponse {
    pub pid: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub location: Option<String>,
    pub is_active: bool,
    pub is_blocked: bool,
    pub is_superuser: bool,
    pub created_at: String,
}

impl AdminUserResponse {
    #[must_use]
    pub fn new(user: &users::Model) -> Self {
        Self {
            pid: user.pid.to_string(),
            email: user.email.clone(),
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
            phone: user.phone.clone(),
            location: user.location.clone(),
            is_active: user.is_active,
            is_blocked: user.is_blocked,
            is_superuser: user.is_superuser,
            created_at: user.created_at.to_string(),
        }
    }
}
