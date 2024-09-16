#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]

use loco_rs::prelude::*;
use axum::debug_handler;
use axum::http::StatusCode;
use loco_rs::controller::ErrorDetail;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::models::_entities::{
    products::{Entity as ProductEntity, Model as ProductModel},
    users,
    wishlists::{self, ActiveModel}};
use crate::views::{
    base::BaseResponse,
};

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct WishListPostParams {
    pub product_id: i32,
    pub user_id: i32,
}

impl WishListPostParams {
    pub(crate) fn update(&self, wishlist: &mut ActiveModel) {
        wishlist.product_id = Set(self.product_id);
    }
}

async fn load_product(ctx: &AppContext, id: i32) -> Result<ProductModel> {
    let msg_error = String::from("Product not found!");
    let product = ProductEntity::find_by_id(id).one(&ctx.db).await?;
    product.ok_or_else(|| Error::CustomError(StatusCode::NOT_FOUND, ErrorDetail::new("not_found", &*msg_error)))
}

#[utoipa::path(
    get,
    path = "/api/user/wishlists",
    tag = "wishlists",
    responses(
        (status = 200, description = "Get all wishlist successfully", body = [ProductsResponse]),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
        (status = 404, description = "Product not found", body = UnauthorizedResponse),
    ),
    security(
        ("jwt_token" = [])
    )
)]
#[debug_handler]
pub async fn user_wishlist_list(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let wishlists = wishlists::Model::get_wishlist_by_user_id(&ctx.db, &user.id).await?;
    format::json(wishlists)
}

#[utoipa::path(
    post,
    path = "/api/user/wishlists/new",
    tag = "wishlists",
    request_body = WishListPostParams,
    responses(
        (status = 200, description = "Add wishlist successfully", body = [BaseResponse], example=json!({"status": "success", "message": "Successfully added into wishlist"})),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
        (status = 404, description = "Product not found", body = UnauthorizedResponse),
    ),
    security(
        ("jwt_token" = [])
    )
)]
#[debug_handler]
pub async fn user_wishlist_new(auth: auth::JWT, State(ctx): State<AppContext>, Json(params): Json<WishListPostParams>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    load_product(&ctx, params.product_id).await?;
    let mut wishlist = ActiveModel {
        user_id: ActiveValue::Set(user.id),
        ..Default::default()
    };
    params.update(&mut wishlist);
    wishlist.insert(&ctx.db).await?;
    let message = "Successfully added into wishlist";
    format::json(BaseResponse::new(&"success".to_string(), &message.to_string()))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/user/wishlists")
        .add("/", get(user_wishlist_list))
        .add("/new", post(user_wishlist_new))
}
