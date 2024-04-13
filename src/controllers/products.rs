#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]

use loco_rs::prelude::*;
use sea_orm::prelude::Decimal;
use serde::{Deserialize, Serialize};
use sea_orm::{ActiveValue, ColumnTrait, QueryFilter, JsonValue};
use chrono::offset::Local;

use crate::models::_entities::{
    products::{self, ActiveModel, Entity, Model},
    users,
    sea_orm_active_enums::Condition,
};
use crate::views::product::ProductResponse;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    pub category: String,
    pub title: String,
    pub description: String,
    pub price: Decimal,
    pub dimension_width: f32,
    pub dimension_height: f32,
    pub dimension_length: f32,
    pub dimension_weight: f32,
    pub brand: String,
    pub material: String,
    pub stock: i32,
    pub sku: String,
    pub seller_id: Option<i32>,
    pub tags: Option<JsonValue>,
    pub condition: Option<Condition>,
    pub images: Option<JsonValue>,
}

impl Params {
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

async fn load_item(ctx: &AppContext, user: users::Model, id: i32) -> Result<Model> {
    let item = user.find_related(Entity).filter(products::Column::Id.eq(id)).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

pub async fn list(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Json<Vec<ProductResponse>>> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let products = user.find_related(Entity).all(&ctx.db).await?;

    let mut product_list: Vec<ProductResponse> = Vec::new();
    for product in &products {
        product_list.push(ProductResponse::new(product));
    }
    format::json(product_list)
}

pub async fn add(auth: auth::JWT, State(ctx): State<AppContext>, Json(params): Json<Params>) -> Result<Json<ProductResponse>> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let mut item = ActiveModel {
        seller_id: ActiveValue::Set(user.id),
        ..Default::default()
    };
    params.update(&mut item);
    let item = item.insert(&ctx.db).await?;
    format::json(ProductResponse::new(&item))
}

pub async fn update(
    auth: auth::JWT,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<Params>,
) -> Result<Json<ProductResponse>> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let item = load_item(&ctx, user, id).await?;
    let mut item = item.into_active_model();
    params.update(&mut item);
    item.updated_at = ActiveValue::Set(Local::now().naive_local());
    let item = item.update(&ctx.db).await?;
    format::json(ProductResponse::new(&item))
}

pub async fn remove(auth: auth::JWT, Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<()> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    load_item(&ctx, user, id).await?.delete(&ctx.db).await?;
    format::empty()
}

pub async fn get_one(auth: auth::JWT, Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Json<ProductResponse>> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let product = load_item(&ctx, user, id).await?;
    format::json(ProductResponse::new(&product))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/products")
        .add("/", get(list))
        .add("/", post(add))
        .add("/:id", get(get_one))
        .add("/:id", delete(remove))
        .add("/:id", post(update))
}