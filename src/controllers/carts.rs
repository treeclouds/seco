#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::_entities::carts::{ActiveModel, Entity, Model};
use crate::models::_entities::users;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    pub user_id: i32,
    pub product_id: i32,
    pub qty: i32,
}

impl Params {
    fn update(&self, item: &mut ActiveModel) {
        item.user_id = Set(self.user_id);
        item.product_id = Set(self.product_id);
        item.qty = Set(self.qty);
    }
}

async fn load_item(ctx: &AppContext, id: i32) -> Result<Model> {
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

#[debug_handler]
pub async fn user_cart_list(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let carts = Model::get_cart_by_user_id(&ctx.db, &user.id).await?;
    format::json(carts)
}

#[debug_handler]
pub async fn user_cart_add(State(ctx): State<AppContext>, Json(params): Json<Params>) -> Result<Response> {
    let mut item = ActiveModel {
        ..Default::default()
    };
    params.update(&mut item);
    let item = item.insert(&ctx.db).await?;
    format::json(item)
}

#[debug_handler]
pub async fn user_cart_update(
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<Params>,
) -> Result<Response> {
    let item = load_item(&ctx, id).await?;
    let mut item = item.into_active_model();
    params.update(&mut item);
    let item = item.update(&ctx.db).await?;
    format::json(item)
}

#[debug_handler]
pub async fn user_cart_remove(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    load_item(&ctx, id).await?.delete(&ctx.db).await?;
    format::empty()
}

#[debug_handler]
pub async fn get_user_cart_one(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    format::json(load_item(&ctx, id).await?)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/user/carts/")
        .add("/", get(user_cart_list))
        .add("/", post(user_cart_add))
        .add("{id}", get(get_user_cart_one))
        .add("{id}", delete(user_cart_remove))
        .add("{id}", put(user_cart_update))
        .add("{id}", patch(user_cart_update))
}
