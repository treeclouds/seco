use loco_rs::prelude::*;

use crate::{models::_entities::users, views::user::CurrentResponse};
use crate::models::products::Entity;
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
    let mut product_list: Vec<ProductResponse> = Vec::new();
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let products = user.find_related(Entity).all(&ctx.db).await?;
    for product in &products {
        product_list.push(ProductResponse::new(product));
    }

    format::json(product_list)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/user")
        .add("/current", get(current))
        .add("/products", get(product_list))
}
