#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::_entities::delivery_method_services::{ActiveModel, Entity, Model};
use crate::models::_entities::users;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    pub delivery_method_id: i32,
    pub name: String,
    pub price: Decimal,
    pub active: bool,
}

impl Params {
    fn update(&self, item: &mut ActiveModel) {
        item.delivery_method_id = Set(self.delivery_method_id);
        item.name = Set(self.name.clone());
        item.price = Set(self.price);
        item.active = Set(self.active);
    }
}

async fn load_item(ctx: &AppContext, id: i32) -> Result<Model> {
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

#[debug_handler]
pub async fn delivery_method_service_list(State(ctx): State<AppContext>) -> Result<Response> {
    format::json(Entity::find().all(&ctx.db).await?)
}

#[debug_handler]
pub async fn delivery_method_service_add(auth: auth::JWT, State(ctx): State<AppContext>, Json(params): Json<Params>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    if !user.is_superuser {
        return bad_request("You are not allowed to add delivery service. Only superuser can do that.");
    }

    let mut item = ActiveModel {
        ..Default::default()
    };
    params.update(&mut item);
    let item = item.insert(&ctx.db).await?;
    format::json(item)
}

#[debug_handler]
pub async fn delivery_method_service_update(
    auth: auth::JWT,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<Params>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    if !user.is_superuser {
        return bad_request("You are not allowed to update delivery service. Only superuser can do that.");
    }
    let item = load_item(&ctx, id).await?;
    let mut item = item.into_active_model();
    params.update(&mut item);
    let item = item.update(&ctx.db).await?;
    format::json(item)
}

#[debug_handler]
pub async fn delivery_method_service_remove(auth: auth::JWT, Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    if !user.is_superuser {
        return bad_request("You are not allowed to delete delivery service. Only superuser can do that.");
    }
    load_item(&ctx, id).await?.delete(&ctx.db).await?;
    format::empty()
}

#[debug_handler]
pub async fn get_delivery_method_service_one(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    format::json(load_item(&ctx, id).await?)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/delivery_method_services/")
        .add("/", get(delivery_method_service_list))
        .add("/new", post(delivery_method_service_add))
        .add("{id}", get(get_delivery_method_service_one))
        .add("{id}", delete(delivery_method_service_remove))
        .add("{id}", put(delivery_method_service_update))
        .add("{id}", patch(delivery_method_service_update))
}
