use crate::models::_entities::materials;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct MaterialResponse {
    pub id: i32,
    pub name: String,
    pub code: Option<String>,
    pub is_active: bool,
}

impl MaterialResponse {
    #[must_use]
    pub fn new(material: &materials::Model) -> Self {
        Self {
            id: material.id,
            name: material.name.clone(),
            code: material.code.clone(),
            is_active: material.is_active,
        }
    }
}
