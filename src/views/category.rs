use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::models::_entities::categories;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CategoryResponse {
    pub id: i32,
    pub name: String,
    pub parent_id: Option<i32>,
}

impl crate::views::category::CategoryResponse {
    #[must_use]
    pub fn new(category: &categories::Model) -> Self {
        Self {
            id: category.id,
            name: category.name.to_string(),
            parent_id: Option::from(category.parent_id),
        }
    }
}