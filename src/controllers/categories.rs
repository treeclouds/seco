#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::models::_entities::categories::{ActiveModel, Model};
use crate::views::category::{CategoryResponse, CategoryListResponse};


#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CategoryPostParams {
    pub name: String,
    pub parent_id: Option<i32>,
}

impl CategoryPostParams {
    pub(crate) fn update(&self, item: &mut ActiveModel) {
        item.name = Set(self.name.clone());
        item.parent_id = Set(self.parent_id.clone());
    }
}

#[utoipa::path(
    get,
    path = "/api/categories",
    tag = "categories",
    responses(
        (status = 200, description = "Categories list", body = [CategoryResponse]),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
    ),
)]
pub async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    let categories: Vec<CategoryListResponse> = Model::get_all_categories(&ctx.db).await?;
    format::json(categories)
}

#[utoipa::path(
    post,
    path = "/api/category/new",
    tag = "categories",
    request_body = CategoryPostParams,
    responses(
        (status = 200, description = "Create a new category successfully", body = CategoryResponse)
    ),
)]
pub async fn category_add(State(ctx): State<AppContext>, Json(params): Json<CategoryPostParams>) -> Result<Response> {
    let mut item = ActiveModel {
        ..Default::default()
    };
    params.update(&mut item);
    let item = item.insert(&ctx.db).await?;
    format::json(CategoryResponse::new(&item))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/categories", get(list))
        .add("/category/new", post(category_add))
}
