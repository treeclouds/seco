#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use sea_orm::prelude::Time;
use serde::{Deserialize, Serialize};

use crate::models::_entities::meetup_addresses::{self, ActiveModel, Entity, Model};
use crate::models::_entities::users;
use sea_orm::{ColumnTrait, QueryFilter};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    /// Ignored: owner is always taken from the JWT.
    #[serde(default)]
    pub user_id: Option<i32>,
    pub date: Date,
    pub time: Time,
    pub city: String,
    pub postal_code: String,
    pub address: String,
    pub notes: Option<String>,
    }

impl Params {
    fn update(&self, item: &mut ActiveModel) {
      item.date = Set(self.date.clone());
      item.time = Set(self.time.clone());
      item.city = Set(self.city.clone());
      item.postal_code = Set(self.postal_code.clone());
      item.address = Set(self.address.clone());
      item.notes = Set(self.notes.clone());
      }
}

async fn current_user(ctx: &AppContext, auth: &auth::JWT) -> Result<users::Model> {
    users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await.map_err(Into::into)
}

/// Load a meetup address owned by `user` (404 if it belongs to someone else).
async fn load_item(ctx: &AppContext, user: &users::Model, id: i32) -> Result<Model> {
    let item = Entity::find_by_id(id)
        .filter(meetup_addresses::Column::UserId.eq(user.id))
        .one(&ctx.db)
        .await?;
    item.ok_or_else(|| Error::NotFound)
}

#[debug_handler]
pub async fn list(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user = current_user(&ctx, &auth).await?;
    format::json(
        Entity::find()
            .filter(meetup_addresses::Column::UserId.eq(user.id))
            .all(&ctx.db)
            .await?,
    )
}

#[debug_handler]
pub async fn add(auth: auth::JWT, State(ctx): State<AppContext>, Json(params): Json<Params>) -> Result<Response> {
    let user = current_user(&ctx, &auth).await?;
    let mut item = ActiveModel {
        user_id: Set(user.id),
        ..Default::default()
    };
    params.update(&mut item);
    let item = item.insert(&ctx.db).await?;
    format::json(item)
}

#[debug_handler]
pub async fn update(
    auth: auth::JWT,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<Params>,
) -> Result<Response> {
    let user = current_user(&ctx, &auth).await?;
    let item = load_item(&ctx, &user, id).await?;
    let mut item = item.into_active_model();
    params.update(&mut item);
    let item = item.update(&ctx.db).await?;
    format::json(item)
}

#[debug_handler]
pub async fn remove(auth: auth::JWT, Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    let user = current_user(&ctx, &auth).await?;
    load_item(&ctx, &user, id).await?.delete(&ctx.db).await?;
    format::empty()
}

#[debug_handler]
pub async fn get_one(auth: auth::JWT, Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    let user = current_user(&ctx, &auth).await?;
    format::json(load_item(&ctx, &user, id).await?)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/meetup_addresses/")
        .add("/", get(list))
        .add("/", post(add))
        .add("{id}", get(get_one))
        .add("{id}", delete(remove))
        .add("{id}", put(update))
        .add("{id}", patch(update))
}
