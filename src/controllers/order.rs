#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use rand::Rng;

use crate::models::_entities::{
    orders::{self, ActiveModel, Entity, Model},
    sea_orm_active_enums::OrderStatusEnum,
    users
};

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct OrderParams {
    #[schema(read_only)]
    pub buyer_id: i32,
    pub order_number: String,
    pub final_price: Decimal,
    #[schema(default = 0)]
    pub delivery_address_id: i32,
    #[schema(read_only)]
    pub delivery_address_detail: Option<String>,
    #[schema(default = 0)]
    pub payment_method_id: i32,
    #[schema(read_only)]
    pub payment_method_detail: Option<serde_json::Value>,
    #[schema(read_only)]
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
    }
}

fn generate_custom_string(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::rng();

    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

async fn load_item(ctx: &AppContext, id: i32) -> Result<Model> {
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

#[utoipa::path(
    get,
    path = "/api/orders",
    tag = "orders",
    responses(
        (status = 200, description = "Set status completed successfully"),
        (status = 401, description = "Bad request"),
    ),
    security(
        ("jwt_token" = [])
    )
)]
#[debug_handler]
pub async fn order_list(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    format::json(Entity::find().filter(orders::Column::BuyerId.eq(user.id)).all(&ctx.db).await?)
}

#[utoipa::path(
    post,
    path = "/api/orders/new",
    tag = "orders",
    responses(
        (status = 200, description = "Create a new order successfully")
    ),
    security(
        ("jwt_token" = [])
    )
)]
#[debug_handler]
pub async fn order_add(auth: auth::JWT, State(ctx): State<AppContext>, Json(params): Json<OrderParams>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let mut item = ActiveModel {
        buyer_id: Set(user.id),
        order_number: Set(generate_custom_string(10)),
        status: Set(OrderStatusEnum::AwaitingPayment),
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

#[utoipa::path(
    get,
    path = "/api/orders/{id}/confirm",
    tag = "orders",
    responses(
        (status = 200, description = "Set status completed successfully"),
        (status = 401, description = "Bad request"),
    ),
    security(
        ("jwt_token" = [])
    )
)]
#[debug_handler]
pub async fn order_confirm(auth: auth::JWT, Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let item = load_item(&ctx, id).await?;

    if item.buyer_id != user.id {
        return Err(Error::BadRequest("Order is not yours".to_string()));
    }

    if item.status != OrderStatusEnum::Shipped {
        return Err(Error::BadRequest("Order is not shipped yet".to_string()));
    }

    item.into_active_model().set_status_completed(&ctx.db).await?;
    format::empty()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/orders/")
        .add("/", get(order_list))
        .add("/new", post(order_add))
        .add("{id}", get(get_order_one))
        .add("{id}", delete(order_remove))
        .add("{id}", put(order_update))
        .add("{id}", patch(order_update))
        .add("{id}/confirm", get(order_confirm))
}
