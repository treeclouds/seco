use chrono::Local;
use loco_rs::prelude::*;

use crate::{models::_entities::{users, products}, views::user::CurrentResponse};
use crate::controllers::products::ProductPostParams;
use crate::models::products::{ActiveModel, Entity, Model};
use crate::views::product::ProductResponse;


#[utoipa::path(
    get,
    path = "/api/user/current",
    tag = "users",
    responses(
        (status = 200, description = "Current user", body = [CurrentResponse]),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
    ),
    security(
        ("jwt_token" = [])
    )
)]
async fn current(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    format::json(CurrentResponse::new(&user))
}

async fn load_item(ctx: &AppContext, user: users::Model, id: i32) -> Result<Model> {
    let item = user.find_related(Entity).filter(products::Column::Id.eq(id)).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

#[utoipa::path(
    get,
    path = "/api/user/products",
    tag = "users",
    responses(
        (status = 200, description = "Product list based on user login successfully", body = [ProductResponse]),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn product_list(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let products = products::Model::get_all_products_by_user_id(&ctx.db, &user.id).await?;
    format::json(products)
}

#[utoipa::path(
    post,
    path = "/api/user/product/new",
    tag = "users",
    request_body = ProductPostParams,
    responses(
        (status = 200, description = "Create a new product successfully", body = ProductResponse)
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn product_add(auth: auth::JWT, State(ctx): State<AppContext>, Json(params): Json<ProductPostParams>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let mut item = ActiveModel {
        seller_id: ActiveValue::Set(user.id),
        ..Default::default()
    };
    params.update(&mut item);
    let item = item.insert(&ctx.db).await?;
    format::json(ProductResponse::new(&item))
}

#[utoipa::path(
    get,
    path = "/api/user/product/{id}",
    tag = "users",
    responses(
        (status = 200, description = "Product detail successfully", body = ProductResponse),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
        (status = 404, description = "Product not found", body = UnauthorizedResponse),
    ),
    params(
        ("id" = i32, Path, description = "Product database id")
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn product_get_one(auth: auth::JWT, Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let product = products::Model::get_product_by_id_and_user_id(&ctx.db, &id, &user.id).await?;
    format::json(product)
}

#[utoipa::path(
    post,
    path = "/api/user/product/{id}",
    tag = "users",
    responses(
        (status = 200, description = "Product update successfully", body = [ProductResponse]),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
        (status = 404, description = "Product not found", body = UnauthorizedResponse),
    ),
    params(
        ("id" = i32, Path, description = "Product database id")
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn product_update(
    auth: auth::JWT,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<ProductPostParams>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let item = load_item(&ctx, user, id).await?;
    let mut item = item.into_active_model();
    params.update(&mut item);
    item.updated_at = ActiveValue::Set(Local::now().naive_local());
    let item = item.update(&ctx.db).await?;
    format::json(ProductResponse::new(&item))
}

#[utoipa::path(
    delete,
    path = "/api/user/product/{id}",
    tag = "users",
    responses(
        (status = 200, description = "Product delete successfully"),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
        (status = 404, description = "Product not found", body = UnauthorizedResponse),
    ),
    params(
        ("id" = i32, Path, description = "Product database id")
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn product_remove(auth: auth::JWT, Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    load_item(&ctx, user, id).await?.delete(&ctx.db).await?;
    format::empty()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/user")
        .add("/current", get(current))
        .add("/products", get(product_list))
        .add("/product/new", post(product_add))
        .add("/product/:id", get(product_get_one))
        .add("/product/:id", delete(product_remove))
        .add("/product/:id", post(product_update))
}
