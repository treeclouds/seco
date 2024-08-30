#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use crate::models::_entities::categories::Entity;
use crate::views::category::CategoryResponse;


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
    let mut category_list: Vec<CategoryResponse> = Vec::new();
    let categories = Entity::find().all(&ctx.db).await?;
    for category in &categories {
        category_list.push(CategoryResponse::new(category));
    }

    format::json(category_list)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/categories", get(list))
}
