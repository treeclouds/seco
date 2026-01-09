#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::_entities::delivery_methods::{ActiveModel, Entity, Model};
use crate::models::_entities::users;
use crate::views::delivery_method::DeliveryMethodsAndServicesResponse;

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct DeliveryMethodParams {
    pub name: String,
    pub active: bool,
}

impl DeliveryMethodParams {
    fn update(&self, item: &mut ActiveModel) {
        item.name = Set(self.name.clone());
        item.active = Set(self.active);
    }
}

async fn load_item(ctx: &AppContext, id: i32) -> Result<Model> {
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

#[utoipa::path(
    get,
    path = "/api/delivery_methods",
    tag = "delivery_methods",
    responses(
        (status = 200, description = "Get all delivery methods successfully", body = [DeliveryMethodsAndServicesResponse]),
    )
)]
#[debug_handler]
pub async fn delivery_method_list(State(ctx): State<AppContext>) -> Result<Response> {
    let delivery_methods = Model::get_all_delivery_methods_and_services(&ctx.db).await?;
    format::json(delivery_methods)
}

#[utoipa::path(
    post,
    path = "/api/delivery_methods/new",
    tag = "delivery_methods",
    request_body = DeliveryMethodParams,
    responses(
        (status = 200, description = "Get all delivery methods successfully", body = [DeliveryMethodsAndServicesResponse]),
    )
)]
#[debug_handler]
pub async fn delivery_method_add(State(ctx): State<AppContext>, Json(params): Json<DeliveryMethodParams>) -> Result<Response> {
    let mut item = ActiveModel {
        ..Default::default()
    };
    params.update(&mut item);
    let item = item.insert(&ctx.db).await?;
    format::json(item)
}

#[debug_handler]
pub async fn delivery_method_update(
    auth: auth::JWT,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<DeliveryMethodParams>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    if !user.is_superuser {
        return bad_request("You are not allowed to update delivery methods. Only superuser can do that.");
    }

    let item = load_item(&ctx, id).await?;
    let mut item = item.into_active_model();
    params.update(&mut item);
    let item = item.update(&ctx.db).await?;
    format::json(item)
}

#[debug_handler]
pub async fn delivery_method_remove(auth: auth::JWT, Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    if !user.is_superuser {
        return bad_request("You are not allowed to delete delivery methods. Only superuser can do that.");
    }
    load_item(&ctx, id).await?.delete(&ctx.db).await?;
    format::empty()
}

#[debug_handler]
pub async fn get_delivery_method_one(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    format::json(load_item(&ctx, id).await?)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/delivery_methods/")
        .add("/", get(delivery_method_list))
        .add("/new", post(delivery_method_add))
        .add("{id}", get(get_delivery_method_one))
        .add("{id}", delete(delivery_method_remove))
        .add("{id}", put(delivery_method_update))
        .add("{id}", patch(delivery_method_update))
}
