use serde::{Deserialize, Serialize};
use sea_orm::{FromQueryResult, JsonValue};
use utoipa::ToSchema;
use crate::models::_entities::categories;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CategoryResponse {
    pub id: i32,
    pub name: String,
    #[schema(default = 0)]
    pub parent_id: Option<i32>,
}

#[derive(Debug, FromQueryResult, Deserialize, Serialize, ToSchema)]
pub struct CategoryListResponse {
    pub id: i32,
    pub name: String,
    pub child: Option<JsonValue>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ParentCategoryResponse {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, FromQueryResult)]
pub struct CategoryTree {
    pub id: i32,
    pub name: String,
    pub parent_id: Option<i32>,
    pub path: String,
}

impl CategoryResponse {
    #[must_use]
    pub fn new(category: &categories::Model) -> Self {
        Self {
            id: category.id,
            name: category.name.to_string(),
            parent_id: Option::from(category.parent_id),
        }
    }
}
