#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use migration::Condition;
use crate::{
    controllers::products::UnauthorizedResponse,
    models::_entities::{
        users,
        orders,
        order_items,
    },
    views::order::OrderDetailResponse,
};

#[utoipa::path(
    get,
    path = "/api/seller/orders",
    tag = "sellers",
    responses(
        (status = 200, description = "Order list based on user login successfully", body = [OrderDetailResponse]),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn seller_order_list(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let orders = orders::Model::get_all_orders_by_seller_id(&ctx.db, &user.id).await?;
    format::json(orders)
}

#[utoipa::path(
    get,
    path = "/api/seller/order/{order_number}",
    tag = "sellers",
    responses(
        (status = 200, description = "Order detail successfully"),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
        (status = 404, description = "Order not found", body = UnauthorizedResponse),
    ),
    params(
        ("order_number" = String, Path, description = "Order database order number")
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn seller_order_detail(auth: auth::JWT, Path(order_number): Path<String>, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let order = orders::Model::get_order_by_seller_id_and_order_number(&ctx.db, &user.id, &order_number).await?;
    format::json(order)
}

#[utoipa::path(
    post,
    path = "/api/seller/order/{order_number}/cancel",
    tag = "sellers",
    responses(
        (status = 200, description = "Order successfully canceled"),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
        (status = 404, description = "Order not found", body = UnauthorizedResponse),
    ),
    params(
        ("order_number" = String, Path, description = "Order database order number")
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn seller_order_cancel(auth: auth::JWT, Path(order_number): Path<String>, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let order = orders::Entity::find()
        .filter(orders::Column::OrderNumber.eq(order_number))
        .one(&ctx.db).await?
        .ok_or_else(|| Error::BadRequest("Order not found".into()))?;

    let is_seller = order_items::Entity::find()
        .filter(
            Condition::all()
                .add(order_items::Column::OrderId.eq(order.id))
                .add(order_items::Column::SellerId.eq(user.id))
        ).one(&ctx.db).await?
        .is_some();

    if !is_seller {
        return Err(Error::BadRequest("Order not found".into()));
    }

    crate::models::orders::cancel_and_restore_stock(&ctx.db, order.id).await?;
    format::empty()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/seller/")
        .add("/orders", get(seller_order_list))
        .add("/order/{order_number}", get(seller_order_detail))
        .add("/order/{order_number}/cancel", post(seller_order_cancel))
}
