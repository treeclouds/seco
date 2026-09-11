#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]

use axum::http::StatusCode;
use loco_rs::controller::ErrorDetail;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use migration::Condition;
use crate::models::_entities::{
    carts::{self, ActiveModel, Entity, Model},
    users,
    products,
};
use crate::controllers::products::UnauthorizedResponse;
use crate::views::{
    base::BaseResponse,
    cart::CartListResponse,
};

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CartPostParams {
    pub product_id: i32,
    pub qty: i32,
}

impl CartPostParams {
    fn update(&self, item: &mut ActiveModel) {
        item.product_id = Set(self.product_id);
        item.qty = Set(self.qty);
    }
}

async fn load_item(ctx: &AppContext, id: i32) -> Result<Model> {
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

async fn load_item_by_product_id(ctx: &AppContext, user_id: i32, product_id: i32) -> Result<Model> {
    let msg_error = String::from("Cart not found!");
    let item = Entity::find()
        .filter(
            Condition::all()
                .add(carts::Column::UserId.eq(user_id))
                .add(carts::Column::ProductId.eq(product_id))
        )
        .one(&ctx.db)
        .await?;
    item.ok_or_else(|| Error::CustomError(StatusCode::NOT_FOUND, ErrorDetail::new("not_found", &*msg_error)))
}

#[utoipa::path(
    get,
    path = "/api/user/carts",
    tag = "carts",
    responses(
        (status = 200, description = "Get all carts successfully", body = [CartListResponse], example = json!({"id": 0, "product_detail": {"id": 0, "title": "string", "images": [], "seller": {}}})),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
        (status = 404, description = "Product not found", body = UnauthorizedResponse),
    ),
    security(
        ("jwt_token" = [])
    )
)]
#[debug_handler]
pub async fn user_cart_list(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let carts = Model::get_cart_by_user_id(&ctx.db, &user.id).await?;
    format::json(carts)
}

#[utoipa::path(
    post,
    path = "/api/user/carts/new",
    tag = "carts",
    request_body(content = CartPostParams, description = "Cart to store the database", content_type = "application/json", example=json!({"product_id": 0})),
    responses(
        (status = 200, description = "Add cart successfully", body = [BaseResponse], example=json!({"status": "success", "message": "Successfully added into cart"})),
        (status = 400, description = "You are not allowed to add your own products to the carts.", body = UnauthorizedResponse),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
        (status = 404, description = "Product not found", body = UnauthorizedResponse),
    ),
    security(
        ("jwt_token" = [])
    )
)]
#[debug_handler]
pub async fn user_cart_add(auth: auth::JWT, State(ctx): State<AppContext>, Json(params): Json<CartPostParams>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let product = products::Entity::find_by_id(params.product_id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;

    if product.seller_id == user.id {
        return Err(Error::BadRequest("You are not allowed to add your own products to the cart".to_string()));
    }

    let mut item = ActiveModel {
        user_id: ActiveValue::Set(user.id),
        ..Default::default()
    };
    params.update(&mut item);
    let item = item.insert(&ctx.db).await?;
    format::json(item)
}

#[debug_handler]
pub async fn user_cart_update(
    auth: auth::JWT,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<CartPostParams>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let item = load_item(&ctx, id).await?;
    if item.user_id != user.id {
        return Err(Error::NotFound);
    }
    let mut item = item.into_active_model();
    params.update(&mut item);
    let item = item.update(&ctx.db).await?;
    format::json(item)
}

#[debug_handler]
pub async fn user_cart_remove(auth: auth::JWT, Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let item = load_item(&ctx, id).await?;
    if item.user_id != user.id {
        return Err(Error::NotFound);
    }
    item.delete(&ctx.db).await?;
    format::empty()
}

#[debug_handler]
pub async fn get_user_cart_one(auth: auth::JWT, Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let item = load_item(&ctx, id).await?;
    if item.user_id != user.id {
        return Err(Error::NotFound);
    }
    format::json(item)
}

#[utoipa::path(
    delete,
    path = "/api/user/carts/{product_id}/remove",
    tag = "carts",
    responses(
        (status = 200, description = "Cart delete successfully"),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
        (status = 404, description = "Cart not found", body = UnauthorizedResponse),
    ),
    params(
        ("product_id" = i32, Path, description = "Product database id")
    ),
    security(
        ("jwt_token" = [])
    )
)]
#[debug_handler]
pub async fn user_cart_delete_by_product_id(auth: auth::JWT, Path(product_id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    // Start checking user validation
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    // End checking user validation

    load_item_by_product_id(&ctx, user.id, product_id).await?.delete(&ctx.db).await?;
    let message = "Cart delete successfully";
    format::json(BaseResponse::new(&"success".to_string(), &message.to_string()))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/user/carts/")
        .add("/", get(user_cart_list))
        .add("/new", post(user_cart_add))
        .add("{id}", get(get_user_cart_one))
        .add("{id}", delete(user_cart_remove))
        .add("{id}", put(user_cart_update))
        .add("{id}", patch(user_cart_update))
        .add("/{product_id}/remove", delete(user_cart_delete_by_product_id))
}
