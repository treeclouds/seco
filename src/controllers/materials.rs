#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::_entities::materials::{ActiveModel, Entity, Model};
use crate::views::materials::MaterialResponse;

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct MaterialParams {
    pub name: String,
    pub code: Option<String>,
    pub is_active: bool,
}

impl MaterialParams {
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
    path = "/api/materials",
    tag = "materials",
    responses(
        (status = 200, description = "Materials list"),
    ),
)]
#[debug_handler]
pub async fn material_list(State(ctx): State<AppContext>) -> Result<Response> {
    format::json(Entity::find().all(&ctx.db).await?)
}

#[utoipa::path(
    post,
    path = "/api/material/new",
    tag = "materials",
    request_body = MaterialParams,
    responses(
        (status = 200, description = "Create a new material successfully", body = MaterialResponse)
    ),
    security(
        ("jwt_token" = [])
    )
)]
#[debug_handler]
pub async fn material_add(State(ctx): State<AppContext>, Json(params): Json<MaterialParams>) -> Result<Response> {
    let mut item = ActiveModel {
        ..Default::default()
    };
    params.update(&mut item);
    let item = item.insert(&ctx.db).await?;
    format::json(MaterialResponse::new(&item))
}

#[debug_handler]
pub async fn material_update(
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<MaterialParams>,
) -> Result<Response> {
    let item = load_item(&ctx, id).await?;
    let mut item = item.into_active_model();
    params.update(&mut item);
    let item = item.update(&ctx.db).await?;
    format::json(item)
}

#[debug_handler]
pub async fn material_remove(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    load_item(&ctx, id).await?.delete(&ctx.db).await?;
    format::empty()
}

#[utoipa::path(
    get,
    path = "/api/material/{id}",
    tag = "materials",
    responses(
        (status = 200, description = "Create a new material successfully", body = MaterialResponse)
    ),
    params(
        ("id" = i32, Path, description = "Material database id")
    )
)]
#[debug_handler]
pub async fn get_material_one(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    let item = load_item(&ctx, id).await?;
    format::json(MaterialResponse::new(&item))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/materials", get(material_list))
        .add("/material/new", post(material_add))
        .add("/material/{id}", get(get_material_one))
        .add("/material/{id}", delete(material_remove))
        .add("/material/{id}", put(material_update))
        .add("/material/{id}", patch(material_update))
}
