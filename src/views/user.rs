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
