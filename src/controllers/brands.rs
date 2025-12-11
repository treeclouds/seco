#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::_entities::brands::{ActiveModel, Entity, Model};
use crate::views::brands::BrandResponse;

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct BrandParams {
    pub name: String,
    pub code: Option<String>,
    pub is_active: bool,
}

impl BrandParams {
    fn update(&self, item: &mut ActiveModel) {
        item.name = Set(self.name.clone());
        item.code = Set(self.code.clone());
        item.is_active = Set(self.is_active);
    }
}

async fn load_item(ctx: &AppContext, id: i32) -> Result<Model> {
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

#[utoipa::path(
    get,
    path = "/api/brands",
    tag = "brands",
    responses(
        (status = 200, description = "Brands list"),
    ),
)]
#[debug_handler]
pub async fn brand_list(State(ctx): State<AppContext>) -> Result<Response> {
    format::json(Entity::find().all(&ctx.db).await?)
}

#[utoipa::path(
    post,
    path = "/api/brand/new",
    tag = "brands",
    request_body = BrandParams,
    responses(
        (status = 200, description = "Create a new brand successfully", body = BrandResponse)
    )
)]
#[debug_handler]
pub async fn brand_add(State(ctx): State<AppContext>, Json(params): Json<BrandParams>) -> Result<Response> {
    let mut item = ActiveModel {
        ..Default::default()
    };
    params.update(&mut item);
    let item = item.insert(&ctx.db).await?;
    format::json(BrandResponse::new(&item))
}

#[debug_handler]
pub async fn brand_update(
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<BrandParams>,
) -> Result<Response> {
    let item = load_item(&ctx, id).await?;
    let mut item = item.into_active_model();
    params.update(&mut item);
    let item = item.update(&ctx.db).await?;
    format::json(BrandResponse::new(&item))
}

#[debug_handler]
pub async fn brand_remove(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    load_item(&ctx, id).await?.delete(&ctx.db).await?;
    format::empty()
}

#[utoipa::path(
    get,
    path = "/api/brand/{id}",
    tag = "brands",
    responses(
        (status = 200, description = "Create a new brand successfully", body = BrandResponse)
    ),
    params(
        ("id" = i32, Path, description = "Brand database id")
    )
)]
#[debug_handler]
pub async fn get_brand_one(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    let item = load_item(&ctx, id).await?;
    format::json(BrandResponse::new(&item))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/brands", get(brand_list))
        .add("/brand/new", post(brand_add))
        .add("/brand/{id}", get(get_brand_one))
        .add("/brand/{id}", delete(brand_remove))
        .add("/brand/{id}", put(brand_update))
        .add("/brand/{id}", patch(brand_update))
}
