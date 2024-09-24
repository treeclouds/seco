#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use axum::debug_handler;
use chrono::Local;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::models::_entities::{
    users,
    sea_orm_active_enums::{ActionType, OfferingStatus}
};
use crate::models::{
    _entities::offerings::{self, ActiveModel, Model},
    products::{self, Model as ProductModel}
};
use crate::views::{
    offering::AddNegotiationProductResponse,
};

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct AddNegotiationProductParams {
    #[schema(value_type = String)]
    pub action: Option<ActionType>,
    #[schema(value_type = f64)]
    pub offer: Decimal,
    pub firebase_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct DoParams {
    pub status: Option<OfferingStatus>
}

impl DoParams {
    pub(crate) fn update(&self, offering: &mut ActiveModel) {
        offering.status = Set(self.status.clone());
    }
}

async fn load_item(ctx: &AppContext, id: i32) -> Result<ProductModel> {
    let item = products::Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

async fn load_offering(ctx: &AppContext, id: i32) -> Result<Model> {
    let offering = offerings::Entity::find_by_id(id).one(&ctx.db).await?;
    offering.ok_or_else(|| Error::NotFound)
}

#[utoipa::path(
    post,
    path = "/api/offering/negotiations/{product_id}",
    tag = "offerings",
    request_body = AddNegotiationProductParams,
    responses(
        (status = 200, description = "Add negotiation successfully", body = [AddNegotiationProductResponse]),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
        (status = 404, description = "Product not found", body = UnauthorizedResponse),
    ),
    params(
        ("product_id" = i32, Path, description = "Product database id")
    ),
    security(
        ("jwt_token" = [])
    )
)]
#[debug_handler]
pub async fn add_negotiation_product(auth: auth::JWT, Path(product_id): Path<i32>, State(ctx): State<AppContext>, Json(params): Json<AddNegotiationProductParams>) -> Result<Response> {
    // Start checking user validation
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    // End checking user validation

    let product = load_item(&ctx, product_id).await?;

    let offering = ActiveModel {
        product_id: ActiveValue::Set(product.id),
        user_id: ActiveValue::Set(user.id),
        product_name: ActiveValue::Set(product.title),
        product_condition: ActiveValue::Set(product.condition),
        product_price: ActiveValue::Set(product.price),
        offer_price: ActiveValue::Set(params.offer),
        action_type: ActiveValue::Set(Option::from(params.action)),
        firebase_id: ActiveValue::Set(Option::from(params.firebase_id)),
        ..Default::default()
    };

    let offering = offering.insert(&ctx.db).await?;
    format::json(AddNegotiationProductResponse::new(&offering))
}

#[utoipa::path(
    post,
    path = "/api/offering/negotiations/{id}/do",
    tag = "offerings",
    request_body = DoParams,
    responses(
        (status = 200, description = "Add wishlist successfully", body = [AddNegotiationProductResponse], example=json!({"status": "Accepted/Declined"})),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
        (status = 404, description = "Product not found", body = UnauthorizedResponse),
    ),
    params(
        ("id" = i32, Path, description = "Offering database id")
    ),
    security(
        ("jwt_token" = [])
    )
)]
#[debug_handler]
pub async fn do_negotiation_product(auth: auth::JWT, Path(id): Path<i32>, State(ctx): State<AppContext>, Json(params): Json<DoParams>) -> Result<Response> {
    // Start checking user validation
    users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    // End checking user validation

    let offering = load_offering(&ctx, id).await?;

    let mut offering = offering.into_active_model();
    params.update(&mut offering);
    offering.updated_at = ActiveValue::Set(DateTimeWithTimeZone::from(Local::now()));
    let offering = offering.update(&ctx.db).await?;

    format::json(AddNegotiationProductResponse::new(&offering))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/offering")
        .add("/negotiations/:product_id", post(add_negotiation_product))
        .add("/negotiations/:id/do", post(do_negotiation_product))
}