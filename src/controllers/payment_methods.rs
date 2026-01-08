#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::{ToSchema};

use crate::models::_entities::{
    sea_orm_active_enums::{PaymentMethodGatewayEnum, PaymentMethodNameEnum},
    payment_methods::{ActiveModel, Entity, Model},
    users,
};
use crate::views::payment_method::PaymentMethodResponse;
use crate::controllers::products::UnauthorizedResponse;

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct PaymentMethodParams {
    #[schema(value_type = String, default = "COD")]
    pub name: PaymentMethodNameEnum,
    pub description: Option<String>,
    #[schema(value_type = i32, default = 60)]
    pub expiry_time: i32,
    #[schema(value_type = String)]
    pub payment_gateway: Option<PaymentMethodGatewayEnum>,
    #[schema(value_type = bool, default = true)]
    pub active: bool,
}

impl PaymentMethodParams {
    fn update(&self, item: &mut ActiveModel) {
        item.name = Set(self.name.to_owned());
        item.description = Set(self.description.clone());
        item.expiry_time = Set(self.expiry_time);
        item.payment_gateway = Set(self.payment_gateway.clone());
        item.active = Set(self.active);
    }
}

async fn load_item(ctx: &AppContext, id: i32) -> Result<Model> {
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

#[utoipa::path(
    get,
    path = "/api/payment_methods",
    tag = "payment_methods",
    responses(
        (status = 200, description = "Get all payment methods successfully", body = [PaymentMethodResponse]),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
        (status = 404, description = "Payment methods not found", body = UnauthorizedResponse),
    ),
    security(
        ("jwt_token" = [])
    )
)]
#[debug_handler]
pub async fn payment_method_list(State(ctx): State<AppContext>) -> Result<Response> {
    let payment_methods: Vec<Model> = Entity::find().all(&ctx.db).await?;
    format::json(payment_methods)
}

#[utoipa::path(
    post,
    path = "/api/payment_methods/new",
    tag = "payment_methods",
    request_body = PaymentMethodParams,
    responses(
        (status = 200, description = "Create a new delivery address successfully", body = PaymentMethodResponse)
    ),
    security(
        ("jwt_token" = [])
    )
)]
#[debug_handler]
pub async fn payment_method_add(auth: auth::JWT, State(ctx): State<AppContext>, Json(params): Json<PaymentMethodParams>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    if !user.is_superuser {
        return bad_request("You are not allowed to update payment methods. Only superuser can do that.");
    }

    let mut item = ActiveModel {
        ..Default::default()
    };
    params.update(&mut item);
    let item = match item.insert(&ctx.db).await {
        Ok(model) => model,
        Err(e) => {
            tracing::error!("DB error lain saat insert item: {:?}", e);
            return bad_request(e.to_string());
        }
    };
    format::json(PaymentMethodResponse::new(&item))
}

#[debug_handler]
pub async fn payment_method_update(
    auth: auth::JWT,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<PaymentMethodParams>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    if !user.is_superuser {
        return bad_request("You are not allowed to update payment methods. Only superuser can do that.");
    }
    let item = load_item(&ctx, id).await?;
    let mut item = item.into_active_model();
    params.update(&mut item);
    let item = item.update(&ctx.db).await?;
    format::json(item)
}

#[debug_handler]
pub async fn payment_method_remove(auth: auth::JWT, Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    if !user.is_superuser {
        return bad_request("You are not allowed to update payment methods. Only superuser can do that.");
    }
    load_item(&ctx, id).await?.delete(&ctx.db).await?;
    format::empty()
}

#[debug_handler]
pub async fn get_payment_method_one(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    format::json(load_item(&ctx, id).await?)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/payment_methods/")
        .add("/", get(payment_method_list))
        .add("/new", post(payment_method_add))
        .add("{id}", get(get_payment_method_one))
        .add("{id}", delete(payment_method_remove))
        .add("{id}", put(payment_method_update))
        .add("{id}", patch(payment_method_update))
}
