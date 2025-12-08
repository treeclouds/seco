#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::_entities::{
    orders::{ActiveModel, Entity, Model},
    sea_orm_active_enums::OrderStatusEnum,
    users,
};

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct OrderParams {
    #[schema(read_only)]
    pub buyer_id: i32,
    pub order_number: String,
    pub final_price: Decimal,
    pub delivery_address_id: i32,
    pub delivery_address_detail: Option<String>,
    pub payment_method_id: i32,
    pub payment_method_detail: Option<serde_json::Value>,
    pub status: OrderStatusEnum,
}

impl OrderParams {
    fn update(&self, item: &mut ActiveModel) {
        item.order_number = Set(self.order_number.clone());
        item.final_price = Set(self.final_price);
        item.delivery_address_id = Set(self.delivery_address_id);
        item.delivery_address_detail = Set(self.delivery_address_detail.clone());
        item.payment_method_id = Set(self.payment_method_id);
        item.payment_method_detail = Set(self.payment_method_detail.clone());
        item.status = Set(self.status.clone());
    }
}

async fn load_item(ctx: &AppContext, id: i32) -> Result<Model> {
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

#[debug_handler]
pub async fn order_list(State(ctx): State<AppContext>) -> Result<Response> {
    format::json(Entity::find().all(&ctx.db).await?)
}

#[debug_handler]
pub async fn order_add(auth: auth::JWT, State(ctx): State<AppContext>, Json(params): Json<OrderParams>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let mut item = ActiveModel {
        buyer_id: Set(user.id),
        ..Default::default()
    };
    params.update(&mut item);
    let item = item.insert(&ctx.db).await?;
    format::json(item)
}

#[debug_handler]
pub async fn order_update(
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<OrderParams>,
) -> Result<Response> {
    let item = load_item(&ctx, id).await?;
    let mut item = item.into_active_model();
    params.update(&mut item);
    let item = item.update(&ctx.db).await?;
    format::json(item)
}

#[debug_handler]
pub async fn order_remove(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    load_item(&ctx, id).await?.delete(&ctx.db).await?;
    format::empty()
}

#[debug_handler]
pub async fn get_order_one(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    format::json(load_item(&ctx, id).await?)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/orders/")
        .add("/", get(order_list))
        .add("/", post(order_add))
        .add("{id}", get(get_order_one))
        .add("{id}", delete(order_remove))
        .add("{id}", put(order_update))
        .add("{id}", patch(order_update))
}
