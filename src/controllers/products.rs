#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]

use loco_rs::prelude::*;
use sea_orm::prelude::Decimal;
use serde::{Deserialize, Serialize};
use sea_orm::{JsonValue};
use utoipa::ToSchema;
use crate::models::_entities::{
    products::{ActiveModel, Model},
    sea_orm_active_enums::Condition,
};
use crate::views::product::ProductsResponse;


#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct ProductPostParams {
    pub category_id: Option<i32>,
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
    pub(crate) fn update(&self, item: &mut ActiveModel) {
        item.title = Set(self.title.clone());
        item.category_id = Set(Option::from(self.category_id));
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

async fn load_item(ctx: &AppContext, id: i32) -> std::result::Result<ProductsResponse, Error> {
    let item = Model::get_product_by_id(&ctx.db, &id).await?;
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
    let products: Vec<ProductsResponse> = Model::get_all_products(&ctx.db).await?;
    format::json(products)
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
    )
)]
pub async fn get_one(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    let product = load_item(&ctx, id).await?;
    format::json(product)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/products", get(list))
        .add("/product/:id", get(get_one))
}