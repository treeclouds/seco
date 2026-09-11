#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::{ToSchema};

use crate::models::_entities::{
    users,
    delivery_addresses::{ActiveModel, Entity, Model}
};
use crate::views::delivery_address::DeliveryAddressResponse;
use crate::controllers::products::UnauthorizedResponse;

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct DeliveryAddressParams {
    #[schema(read_only)]
    pub user_id: Option<i32>,
    pub name: String,
    pub phone: String,
    pub email: String,
    pub city: String,
    pub postal_code: String,
    pub address: String,
    pub notes: Option<String>,
    pub latitude: Option<String>,
    pub longitude: Option<String>,
}

impl DeliveryAddressParams {
    fn update(&self, item: &mut ActiveModel) {
        item.name = Set(self.name.clone());
        item.phone = Set(self.phone.clone());
        item.email = Set(self.email.clone());
        item.city = Set(self.city.clone());
        item.postal_code = Set(self.postal_code.clone());
        item.address = Set(self.address.clone());
        item.notes = Set(self.notes.clone());
        item.latitude = Set(self.latitude.clone());
        item.longitude = Set(self.longitude.clone());
    }
}

#[utoipa::path(
    get,
    path = "/api/user/delivery_addresses",
    tag = "delivery_addresses",
    responses(
        (status = 200, description = "Get all delivery address successfully", body = [DeliveryAddressResponse]),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
        (status = 404, description = "Delivery address not found", body = UnauthorizedResponse),
    ),
    security(
        ("jwt_token" = [])
    )
)]
#[debug_handler]
pub async fn delivery_address_list(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let delivery_addresses: Vec<Model> = user.find_related(Entity).all(&ctx.db).await?;
    format::json(delivery_addresses)
}

#[utoipa::path(
    post,
    path = "/api/user/delivery_address/new",
    tag = "delivery_addresses",
    request_body = DeliveryAddressParams,
    responses(
        (status = 200, description = "Create a new delivery address successfully", body = DeliveryAddressResponse)
    ),
    security(
        ("jwt_token" = [])
    )
)]
#[debug_handler]
pub async fn delivery_address_add(auth: auth::JWT, State(ctx): State<AppContext>, Json(params): Json<DeliveryAddressParams>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let mut item = ActiveModel {
        user_id: ActiveValue::set(user.id),
        ..Default::default()
    };
    params.update(&mut item);
    let item = item.insert(&ctx.db).await?;
    format::json(DeliveryAddressResponse::new(&item))
}

#[utoipa::path(
    put,
    path = "/api/user/delivery_address/{id}",
    tag = "delivery_addresses",
    request_body = DeliveryAddressParams,
    responses(
        (status = 200, description = "Delivery address update successfully", body = DeliveryAddressResponse),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
        (status = 404, description = "Delivery address not found", body = UnauthorizedResponse),
    ),
    params(
        ("id" = i32, Path, description = "Delivery address database id")
    ),
    security(
        ("jwt_token" = [])
    )
)]
#[debug_handler]
pub async fn delivery_address_update(
    auth: auth::JWT,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<DeliveryAddressParams>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let Ok(delivery_address) = Model::find_by_id_and_user_id(&ctx.db, id, user.id).await else {
        let msg_error = String::from("Not found delivery address with this id and user id");
        return bad_request(&msg_error);
    };
    let mut item = delivery_address.into_active_model();
    params.update(&mut item);
    let item = item.update(&ctx.db).await?;
    format::json(DeliveryAddressResponse::new(&item))
}

#[utoipa::path(
    delete,
    path = "/api/user/delivery_address/{id}",
    tag = "delivery_addresses",
    responses(
        (status = 200, description = "Delivery address delete successfully"),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
        (status = 404, description = "Delivery address not found", body = UnauthorizedResponse),
    ),
    params(
        ("id" = i32, Path, description = "Delivery address database id")
    ),
    security(
        ("jwt_token" = [])
    )
)]
#[debug_handler]
pub async fn delivery_address_remove(auth: auth::JWT, Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let Ok(delivery_address) = Model::find_by_id_and_user_id(&ctx.db, id, user.id).await else {
        let msg_error = String::from("Not found delivery address with this id and user id");
        return bad_request(&msg_error);
    };
    delivery_address.delete(&ctx.db).await?;
    format::empty()
}

#[utoipa::path(
    get,
    path = "/api/user/delivery_address/{id}",
    tag = "delivery_addresses",
    responses(
        (status = 200, description = "Delivery address detail successfully", body = [DeliveryAddressResponse]),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
        (status = 404, description = "Delivery address not found", body = UnauthorizedResponse),
    ),
    params(
        ("id" = i32, Path, description = "Delivery address database id")
    ),
    security(
        ("jwt_token" = [])
    )
)]
#[debug_handler]
pub async fn get_delivery_address_one(auth: auth::JWT, Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let Ok(delivery_address) = Model::find_by_id_and_user_id(&ctx.db, id, user.id).await else {
        let msg_error = String::from("Not found delivery address with this id and user id");
        return bad_request(&msg_error);
    };
    format::json(DeliveryAddressResponse::new(&delivery_address))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/user")
        .add("/delivery_addresses", get(delivery_address_list))
        .add("/delivery_address/new", post(delivery_address_add))
        .add("/delivery_address/{id}", get(get_delivery_address_one))
        .add("/delivery_address/{id}", delete(delivery_address_remove))
        .add("/delivery_address/{id}", put(delivery_address_update))
        .add("/delivery_address/{id}", patch(delivery_address_update))
}
