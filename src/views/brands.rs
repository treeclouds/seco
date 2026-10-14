use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::models::_entities::brands;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct BrandResponse {
    pub id: i32,
    pub name: String,
    pub code: Option<String>,
    pub is_active: bool,
}

impl BrandResponse {
    #[must_use]
    pub fn new(brand: &brands::Model) -> Self {
        Self {
            id: brand.id,
            name: brand.name.to_string(),
            code: Option::from(brand.code.clone()),
            is_active: brand.is_active,
        }
    }
}