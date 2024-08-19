#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]

use loco_rs::prelude::*;
use sea_orm::prelude::Decimal;
use serde::{Deserialize, Serialize};
use sea_orm::{ActiveValue, ColumnTrait, QueryFilter, JsonValue};
use chrono::offset::Local;
use utoipa::ToSchema;

use crate::models::_entities::{
    products::{self, ActiveModel, Entity, Model},
    users,
    sea_orm_active_enums::Condition,
};
use crate::views::product::ProductResponse;


#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct ProductPostParams {
    pub category: String,
    pub title: String,
    pub description: String,
    #[schema(value_type = f64)]
    pub price: Decimal,
    pub dimension_width: f32,
    pub dimension_height: f32,
    pub dimension_length: f32,
    pub dimension_weight: f32,
    pub brand: String,
    pub material: String,
    pub stock: i32,
    pub sku: String,
    #[schema(read_only)]
    pub seller_id: Option<i32>,
    #[schema(value_type = String, format = Binary)]
    pub tags: Option<JsonValue>,
    #[schema(value_type = String)]
    pub condition: Option<Condition>,
    #[schema(value_type = String, format = Binary)]
    pub images: Option<JsonValue>,
}

impl ProductPostParams {
    fn update(&self, item: &mut ActiveModel) {
        item.title = Set(self.title.clone());
        item.category = Set(self.category.clone());
        item.description = Set(self.description.clone());
        item.price = Set(self.price);
        item.dimension_width = Set(self.dimension_width);
        item.dimension_height = Set(self.dimension_height);
        item.dimension_length = Set(self.dimension_length);
        item.dimension_weight = Set(self.dimension_weight);
        item.brand = Set(self.brand.clone());
        item.material = Set(self.material.clone());
        item.stock = Set(self.stock);
        item.sku = Set(self.sku.clone());
        item.condition = Set(self.condition.clone());
        item.tags = Set(self.tags.clone());
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct UnauthorizedResponse {
    pub error: String,
    pub description: String,
}

async fn load_item(ctx: &AppContext, user: users::Model, id: i32) -> Result<Model> {
    let item = user.find_related(Entity).filter(products::Column::Id.eq(id)).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

#[utoipa::path(
    get,
    path = "/api/products",
    tag = "products",
    responses(
        (status = 200, description = "Product list based on user login successfully", body = [ProductResponse]),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
    ),
)]
pub async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    let mut product_list: Vec<ProductResponse> = Vec::new();
    // if !&auth.claims.pid.trim().is_empty() {
    //     let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    //     let products = user.find_related(Entity).all(&ctx.db).await?;
    //     for product in &products {
    //         product_list.push(ProductResponse::new(product));
    //     }
    // } else {
    let products = Entity::find().all(&ctx.db).await?;
    for product in &products {
        product_list.push(ProductResponse::new(product));
    }
    // }

    format::json(product_list)
}

#[utoipa::path(
    post,
    path = "/api/product/new",
    tag = "products",
    request_body = ProductPostParams,
    responses(
        (status = 200, description = "Create a new product successfully", body = ProductResponse)
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn add(auth: auth::JWT, State(ctx): State<AppContext>, Json(params): Json<ProductPostParams>) -> Result<Response> {
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
    post,
    path = "/api/product/{id}",
    tag = "products",
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
pub async fn update(
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
    path = "/api/product/{id}",
    tag = "products",
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
pub async fn remove(auth: auth::JWT, Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    load_item(&ctx, user, id).await?.delete(&ctx.db).await?;
    format::empty()
}

#[utoipa::path(
    get,
    path = "/api/product/{id}",
    tag = "products",
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
pub async fn get_one(auth: auth::JWT, Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let product = load_item(&ctx, user, id).await?;
    format::json(ProductResponse::new(&product))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/products", get(list))
        .add("/product/new", post(add))
        .add("/product/:id", get(get_one))
        .add("/product/:id", delete(remove))
        .add("/product/:id", post(update))
}