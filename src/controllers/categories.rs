#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use axum::extract::Query;
use sea_orm::QueryOrder;
use crate::models::_entities::categories::{self, ActiveModel, Model, Entity};
use crate::views::category::{CategoryResponse, CategoryListResponse, ParentCategoryResponse, CategoryTree};
use crate::controllers::products::UnauthorizedResponse;

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CategoryPostParams {
    pub name: String,
    #[schema(default = 0)]
    pub parent_id: Option<i32>,
}

impl CategoryPostParams {
    pub(crate) fn update(&self, item: &mut ActiveModel) {
        item.name = Set(self.name.clone());
        item.parent_id = Set(self.parent_id.clone());
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ListParams {
    pub exclude_id: Option<i32>,
}

async fn load_item(ctx: &AppContext, id: i32) -> Result<Model> {
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

pub async fn load_parents(State(ctx): State<AppContext>, Query(params): Query<ListParams>) -> Result<Response> {
    let mut parent_categories = Entity::find().all(&ctx.db).await?;
    if params.exclude_id.is_some() {
        parent_categories = Entity::find().filter(categories::Column::Id.ne(params.exclude_id)).all(&ctx.db).await?;
    }
    let response: Vec<ParentCategoryResponse> = parent_categories
        .into_iter()
        .map(|c| ParentCategoryResponse {
            id: c.id,
            name: c.name,
        })
        .collect();
    format::json(response)
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
#[debug_handler]
pub async fn get_all_categories(State(ctx): State<AppContext>) -> Result<Response> {
    let categories: Vec<CategoryListResponse> = Model::get_all_categories(&ctx.db).await?;
    format::json(categories)
}

#[debug_handler]
pub async fn get_categories(State(ctx): State<AppContext>) -> Result<Response> {
    // let categories = Entity::find().order_by_asc(categories::Column::Name).all(&ctx.db).await?;
    // let response: Vec<CategoryResponse> = categories
    //     .into_iter()
    //     .map(|c| CategoryResponse::new(&c))
    //     .collect();
    let categories: Vec<CategoryTree> = Model::get_category_tree(&ctx.db).await?;
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
#[debug_handler]
pub async fn category_add(State(ctx): State<AppContext>, Json(params): Json<CategoryPostParams>) -> Result<Response> {
    let mut item = ActiveModel {
        ..Default::default()
    };
    params.update(&mut item);
    let item = item.insert(&ctx.db).await?;
    format::json(CategoryResponse::new(&item))
}

#[debug_handler]
pub async fn get_category_one(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    let item = load_item(&ctx, id).await?;
    format::json(CategoryResponse::new(&item))
}

#[debug_handler]
pub async fn category_update(
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<CategoryPostParams>,
) -> Result<Response> {
    let item = load_item(&ctx, id).await?;
    let mut item = item.into_active_model();
    params.update(&mut item);
    let item = item.update(&ctx.db).await?;
    format::json(CategoryResponse::new(&item))
}

pub async fn category_remove(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    load_item(&ctx, id).await?.delete(&ctx.db).await?;
    format::empty()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/categories", get(get_all_categories))
        .add("/get_categories", get(get_categories))
        .add("/parent_categories", get(load_parents))
        .add("/category/new", post(category_add))
        .add("/category/{id}", get(get_category_one))
        .add("/category/{id}", delete(category_remove))
        .add("/category/{id}", put(category_update))
        .add("/category/{id}", patch(category_update))
}
