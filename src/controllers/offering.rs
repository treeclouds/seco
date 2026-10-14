#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use sea_orm::QueryOrder;
use loco_rs::controller::ErrorDetail;
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
use crate::controllers::products::UnauthorizedResponse;

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct AddNegotiationProductParams {
    #[schema(value_type = String)]
    pub action: Option<ActionType>,
    #[schema(value_type = f64)]
    pub offer: Decimal,
    pub firebase_id: String,
    pub product_id: i32,
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

/// Who may act on (accept/decline/view) an offering:
/// - offer made by a buyer  -> only the product's seller may accept/decline;
/// - counter-offer made by the seller -> only a buyer who has an offer in the
///   same negotiation (firebase_id) may accept/decline.
/// The creator can never accept their own offer.
async fn can_respond(ctx: &AppContext, actor: &users::Model, offering: &Model, product: &ProductModel) -> Result<bool> {
    if actor.id == offering.user_id {
        return Ok(false);
    }
    if offering.user_id != product.seller_id {
        return Ok(actor.id == product.seller_id);
    }
    let Some(fid) = offering.firebase_id.clone() else { return Ok(false) };
    let buyer_offer = offerings::Entity::find()
        .filter(offerings::Column::FirebaseId.eq(fid))
        .filter(offerings::Column::ProductId.eq(product.id))
        .filter(offerings::Column::UserId.eq(actor.id))
        .one(&ctx.db)
        .await?;
    Ok(buyer_offer.is_some())
}

async fn is_party(ctx: &AppContext, actor: &users::Model, offering: &Model) -> Result<bool> {
    if actor.id == offering.user_id {
        return Ok(true);
    }
    let product = load_item(ctx, offering.product_id).await?;
    can_respond(ctx, actor, offering, &product).await
}

async fn load_offering_by_firebase_id(ctx: &AppContext, firebase_id: String) -> Result<Model> {
    let offering = offerings::Entity::find()
        .filter(offerings::Column::FirebaseId.eq(firebase_id))
        .order_by_desc(offerings::Column::Id)
        .one(&ctx.db)
        .await?;
    offering.ok_or_else(|| Error::NotFound)
}

#[utoipa::path(
    post,
    path = "/api/offering/negotiations/new",
    tag = "offerings",
    request_body = AddNegotiationProductParams,
    responses(
        (status = 200, description = "Add negotiation successfully", body = [AddNegotiationProductResponse]),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
        (status = 404, description = "Product not found", body = UnauthorizedResponse),
    ),
    security(
        ("jwt_token" = [])
    )
)]
#[debug_handler]
pub async fn add_negotiation_product(auth: auth::JWT, State(ctx): State<AppContext>, Json(params): Json<AddNegotiationProductParams>) -> Result<Response> {
    // Start checking user validation
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    // End checking user validation

    let product = load_item(&ctx, params.product_id).await?;

    if params.offer <= Decimal::ZERO {
        return bad_request("offer must be greater than 0");
    }
    let firebase_id = params.firebase_id.trim().to_string();
    if firebase_id.is_empty() {
        return bad_request("firebase_id is required");
    }
    if product.status == crate::models::_entities::sea_orm_active_enums::ProductStatus::Draft {
        return Err(Error::NotFound);
    }
    let is_seller = product.seller_id == user.id;
    if is_seller && params.action != Some(ActionType::CounterOffer) {
        return bad_request("You cannot make an offer on your own product");
    }

    // A negotiation (firebase_id) belongs to one product and at most one open offer.
    let existing = offerings::Entity::find()
        .filter(offerings::Column::FirebaseId.eq(firebase_id.clone()))
        .all(&ctx.db)
        .await?;
    if existing.iter().any(|o| o.product_id != product.id) {
        return Err(Error::CustomError(
            axum::http::StatusCode::CONFLICT,
            ErrorDetail::new("conflict", "firebase_id already used for another product"),
        ));
    }
    if existing.iter().any(|o| o.status == Some(OfferingStatus::InProgress)) {
        return Err(Error::CustomError(
            axum::http::StatusCode::CONFLICT,
            ErrorDetail::new("conflict", "there is already an open offer in this negotiation"),
        ));
    }
    if is_seller && !existing.iter().any(|o| o.user_id != user.id) {
        return bad_request("Counter-offer requires an existing buyer offer in this negotiation");
    }
    if !is_seller && existing.iter().any(|o| o.user_id != user.id && o.user_id != product.seller_id) {
        return Err(Error::CustomError(
            axum::http::StatusCode::CONFLICT,
            ErrorDetail::new("conflict", "firebase_id belongs to another negotiation"),
        ));
    }

    let offering = ActiveModel {
        product_id: ActiveValue::Set(product.id),
        user_id: ActiveValue::Set(user.id),
        product_name: ActiveValue::Set(product.title),
        product_condition: ActiveValue::Set(product.condition),
        product_price: ActiveValue::Set(product.price),
        offer_price: ActiveValue::Set(params.offer),
        action_type: ActiveValue::Set(Option::from(params.action)),
        firebase_id: ActiveValue::Set(Some(firebase_id)),
        status: ActiveValue::Set(Some(OfferingStatus::InProgress)),
        ..Default::default()
    };

    let offering = offering.insert(&ctx.db).await?;
    format::json(AddNegotiationProductResponse::new(&offering))
}

#[utoipa::path(
    post,
    path = "/api/offering/negotiations/{id}/do",
    tag = "offerings",
    request_body(content = DoParams, description = "Accepted/Declined Offering", content_type = "application/json", example=json!({"status": "Accepted/Declined"})),
    responses(
        (status = 200, description = "Add offering successfully", body = [AddNegotiationProductResponse]),
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
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    // End checking user validation

    let offering = load_offering(&ctx, id).await?;
    let product = load_item(&ctx, offering.product_id).await?;
    if !can_respond(&ctx, &user, &offering, &product).await? {
        return Err(crate::controllers::base::forbidden("You are not allowed to respond to this offer"));
    }
    if offering.status != Some(OfferingStatus::InProgress) {
        return bad_request("Offer is no longer open");
    }
    if !matches!(params.status, Some(OfferingStatus::Accepted | OfferingStatus::Declined)) {
        return bad_request("status must be Accepted or Declined");
    }

    let mut offering = offering.into_active_model();
    params.update(&mut offering);
    offering.updated_at = ActiveValue::Set(DateTimeWithTimeZone::from(Local::now()));
    let offering = offering.update(&ctx.db).await?;

    format::json(AddNegotiationProductResponse::new(&offering))
}

#[utoipa::path(
    get,
    path = "/api/offering/negotiations/{firebase_id}",
    tag = "offerings",
    responses(
        (status = 200, description = "Get offering successfully", body = [AddNegotiationProductResponse]),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
        (status = 404, description = "Product not found", body = UnauthorizedResponse),
    ),
    params(
        ("firebase_id" = i32, Path, description = "Offering database id")
    ),
    security(
        ("jwt_token" = [])
    )
)]
#[debug_handler]
pub async fn get_negotiation(auth: auth::JWT, Path(firebase_id): Path<String>, State(ctx): State<AppContext>) -> Result<Response> {
    // Start checking user validation
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    // End checking user validation

    let offering = load_offering_by_firebase_id(&ctx, firebase_id).await?;
    if !is_party(&ctx, &user, &offering).await? {
        return Err(Error::NotFound);
    }
    return match offering.status {
        Some(OfferingStatus::InProgress) => format::json(AddNegotiationProductResponse::new(&offering)),
        _ => format::json(())
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/offering/negotiations/new", post(add_negotiation_product))
        .add("/offering/negotiations/{firebase_id}", get(get_negotiation))
        .add("/offering/negotiations/{id}/do", post(do_negotiation_product))
}
