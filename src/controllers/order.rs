#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;
use rand::Rng;
use sea_orm::ActiveEnum;

use crate::models::_entities::{
    orders::{self, ActiveModel, Entity, Model},
    sea_orm_active_enums::OrderStatusEnum,
    users,
    delivery_addresses,
    products,
    payment_methods,
    order_items::{ActiveModel as OrderItemActiveModel},
};

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct MeetupAddressDetail {
    pub date: String,
    pub time: String,
    pub city: String,
    pub postal_code: String,
    pub address: String,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct OrderParams {
    #[schema(read_only)]
    pub buyer_id: Option<i32>,
    #[schema(read_only)]
    pub order_number: Option<String>,
    #[schema(read_only)]
    pub final_price: Decimal,
    #[schema(default = 1)]
    pub delivery_address_id: Option<i32>,
    #[schema(read_only)]
    pub delivery_address_detail: Option<String>,
    #[schema(default = 1)]
    pub payment_method_id: i32,
    #[schema(default = 1)]
    pub product_id: i32,
    #[schema(read_only, value_type = String)]
    pub payment_method_detail: Option<serde_json::Value>,
    #[schema(read_only, value_type = String)]
    pub status: Option<OrderStatusEnum>,
    pub meetup_address_detail: Option<MeetupAddressDetail>,
}

impl OrderParams {
    fn update(&self, item: &mut ActiveModel) {
        item.delivery_address_id = Set(self.delivery_address_id);
        item.payment_method_id = Set(self.payment_method_id);
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
    request_body = OrderParams,
    responses(
        (status = 200, description = "Create a new order successfully")
    ),
    security(
        ("jwt_token" = [])
    )
)]
#[debug_handler]
pub async fn order_add(auth: auth::JWT, State(ctx): State<AppContext>, Json(params): Json<OrderParams>) -> Result<Response> {
    let buyer = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let product = products::Entity::find_by_id(params.product_id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::BadRequest("Product not found".into()))?;

    // check product stock
    if product.stock <= 0 {
        let msg_error = String::from("Product is out of stock");
        return bad_request(&msg_error);
    }

    let delivery_address_detail = match params.delivery_address_id {
        Some(d) if d > 0 => {
            let delivery_address = delivery_addresses::Entity::find_by_id(d)
                .one(&ctx.db)
                .await?
                .ok_or_else(|| Error::BadRequest("Delivery address not found".into()))?;

            if delivery_address.user_id != buyer.id {
                let msg_error = String::from("Delivery address is not yours");
                return bad_request(&msg_error);
            }

            Some(json!({
                "name": delivery_address.name,
                "phone": delivery_address.phone,
                "email": delivery_address.email,
                "city": delivery_address.city,
                "address": delivery_address.address,
            }).to_string())
        },
        _ => Some("".to_string()),
    };
    let payment_method = payment_methods::Entity::find_by_id(params.payment_method_id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::BadRequest("Payment method not found".into()))?;

    let payment_gateway = match payment_method.payment_gateway {
        Some(pg) => pg.to_value(),
        None => "-".to_string(),
    };
    let payment_method_detail = json!({
        "name": payment_method.clone().name,
        "expiry_time": payment_method.clone().expiry_time,
        "payment_gateway": payment_gateway,
    });
    let meetup_address_detail = params.meetup_address_detail.as_ref().map(|m| {
        json!({
            "date": m.date,
            "time": m.time,
            "city": m.city,
            "postal_code": m.postal_code,
            "address": m.address,
            "notes": m.notes,
        })
    });
    let meetup_address_detail = meetup_address_detail.map(Into::into);
    let mut order = ActiveModel {
        buyer_id: Set(buyer.id),
        order_number: Set(generate_custom_string(10)),
        status: Set(OrderStatusEnum::AwaitingPayment),
        final_price: Set(product.price),
        delivery_address_detail: Set(delivery_address_detail),
        payment_method_detail: Set(Some(payment_method_detail)),
        meetup_address_detail: Set(meetup_address_detail),
        ..Default::default()
    };
    params.update(&mut order);
    let order = order.insert(&ctx.db).await?;
    let qty = 1;
    let order_item = OrderItemActiveModel {
        order_id: Set(order.id),
        product_name: Set(Some(product.title.clone())),
        product_price: Set(product.price),
        product_sku: Set(Some(product.sku.clone())),
        product_condition: Set(product.condition.as_ref().map(|c| c.to_value())),
        qty: Set(qty),
        seller_id: Set(product.seller_id),
        product_id: Set(params.product_id),
        ..Default::default()
    };
    order_item.insert(&ctx.db).await?;

    // decrease product stock
    product.into_active_model().set_decrease_stock(&ctx.db, qty).await?;

    format::json(order)
}

#[debug_handler]
pub async fn order_update(
    auth: auth::JWT,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<OrderParams>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let item = load_item(&ctx, id).await?;
    if item.buyer_id != user.id {
        return Err(Error::NotFound);
    }
    let mut item = item.into_active_model();
    params.update(&mut item);
    let item = item.update(&ctx.db).await?;
    format::json(item)
}

#[debug_handler]
pub async fn order_remove(auth: auth::JWT, Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let item = load_item(&ctx, id).await?;
    if item.buyer_id != user.id {
        return Err(Error::NotFound);
    }
    item.delete(&ctx.db).await?;
    format::empty()
}

#[debug_handler]
pub async fn get_order_one(auth: auth::JWT, Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let item = load_item(&ctx, id).await?;
    if item.buyer_id != user.id {
        return Err(Error::NotFound);
    }
    format::json(item)
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
