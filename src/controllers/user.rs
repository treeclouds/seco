use chrono::Local;
use std::path::PathBuf;
use axum::extract::Multipart;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use bytes::Bytes;
use sea_orm::ActiveEnum;
use crate::{
    controllers::{
        upload::generate_unique_filename,
        products::{ProductPostParams, UnauthorizedResponse}
    },
    models::_entities::{
        users::{self, ActiveModel},
        products::{self, ActiveModel as ProductActiveModel, Entity as ProductEntity, Model as ProductModel},
        product_images::{ActiveModel as ProductImageActiveModel, Model as ProductImageModel},
        orders::{self, Entity as OrderEntity}
    },
    views::{
        order::OrderResponse,
        product::ProductResponse,
        product_image::ProductImageResponse,
        user::CurrentResponse,
    },
};

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct LocationParams {
    pub location: String,
    pub latitude: Option<String>,
    pub longitude: Option<String>,
}

impl LocationParams {
    pub(crate) fn update(&self, item: &mut ActiveModel) {
        item.location = Set(Option::from(self.location.clone()));
        item.latitude = Set(Option::from(self.latitude.clone()));
        item.longitude = Set(Option::from(self.longitude.clone()));
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct ProfileParams {
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub email: String,
    pub location: String,
    pub latitude: Option<String>,
    pub longitude: Option<String>,
}

impl ProfileParams {
    pub(crate) fn update(&self, item: &mut ActiveModel) {
        item.first_name = Set(self.first_name.clone());
        item.last_name = Set(self.last_name.clone());
        item.phone = Set(Option::from(self.phone.clone()));
        item.email = Set(self.email.clone());
        item.location = Set(Option::from(self.location.clone()));
        item.latitude = Set(Option::from(self.latitude.clone()));
        item.longitude = Set(Option::from(self.longitude.clone()));
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateProductMultipart {
    /// JSON string berisi data produk, sesuai `ProductPostParams`
    ///
    /// Example:
    /// `{"title":"Produk A","price":10000}`
    // #[schema(value_type = String, example = r#"{"title":"Produk A","price":10000}"#)]
    pub product: ProductPostParams,

    /// File gambar produk (boleh lebih dari satu).
    ///
    /// Secara OpenAPI ini adalah array of binary files.
    #[schema(value_type = [String], format = Binary)]
    pub files: Option<Vec<String>>,
}

struct PendingFile {
    file_name: String,
    content: Bytes,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct ProductWithImagesResponse {
    product: ProductResponse,
    images: Vec<ProductImageResponse>,
}

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

async fn load_item(ctx: &AppContext, user: users::Model, id: i32) -> Result<ProductModel> {
    let item = user.find_related(ProductEntity).filter(products::Column::Id.eq(id)).one(&ctx.db).await?;
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
    request_body(content = CreateProductMultipart, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Create a new product successfully", body = ProductWithImagesResponse)
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn product_add(auth: auth::JWT, State(ctx): State<AppContext>, mut multipart: Multipart) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let mut product_params: Option<ProductPostParams> = None;
    let mut pending_files: Vec<PendingFile> = Vec::new();

    // 1. Read all fields from the multipart request
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| {
            tracing::error!(error = ?err, "could not read multipart");
            Error::BadRequest("could not read multipart".into())
        })?
    {
        let field_name = field.name().map(|s| s.to_string());

        match field_name.as_deref() {
            // The text field contains the product JSON
            Some("product") => {
                let text = field.text().await.map_err(|err| {
                    tracing::error!(error = ?err, "could not read product field as text");
                    Error::BadRequest("could not read product field".into())
                })?;

                let params: ProductPostParams = serde_json::from_str(&text).map_err(|err| {
                    tracing::error!(error = ?err, "invalid product json");
                    Error::BadRequest("invalid product json".into())
                })?;

                product_params = Some(params);
            }

            // The file fields contain the images
            Some("files") => {
                let file_name = match field.file_name() {
                    Some(file_name) => file_name.to_string(),
                    None => {
                        return bad_request("file name not found")
                    }
                };

                let content = field.bytes().await.map_err(|err| {
                    tracing::error!(error = ?err, "could not read file bytes");
                    Error::BadRequest("could not read file bytes".into())
                })?;

                pending_files.push(PendingFile { file_name, content });
            }

            _ => {
                // Ignore other fields
            }
        }
    }

    // 2. Ensure the product payload exists
    let product_params = product_params.ok_or_else(|| {
        Error::BadRequest("product data not found in multipart".into())
    })?;

    // 3. Insert the product first
    let mut item = ProductActiveModel {
        seller_id: Set(user.id),
        ..Default::default()
    };
    product_params.update(&mut item);

    let item = item.insert(&ctx.db).await?;
    let product_id = item.id;

    // 4. Process all uploaded files → upload to storage + save to the database
    let mut images: Vec<ProductImageResponse> = Vec::new();

    for f in pending_files {
        let unique_file_name = generate_unique_filename(&f.file_name, product_id).await?;
        let unique_file_name_str = unique_file_name.to_string_lossy().to_string();
        let new_filename = format!("{product_id}/{unique_file_name_str}");
        let path = PathBuf::from("product_images").join(&new_filename);

        ctx.storage
            .as_ref()
            .upload(path.as_path(), &f.content)
            .await?;

        let product_image = ProductImageActiveModel {
            product_id: Set(product_id),
            image: Set(path.to_string_lossy().into_owned()),
            ..Default::default()
        };

        let product_image: ProductImageModel = product_image.insert(&ctx.db).await?;
        images.push(ProductImageResponse::new(&product_image));
    }

    let resp = ProductWithImagesResponse {
        product: ProductResponse::new(&item),
        images,
    };

    format::json(resp)
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

#[utoipa::path(
    post,
    path = "/api/user/update_location",
    tag = "users",
    request_body = LocationParams,
    responses(
        (status = 200, description = "Product update successfully", body = [CurrentResponse]),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
        (status = 404, description = "Product not found", body = UnauthorizedResponse),
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn update_location(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<LocationParams>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let mut user = user.into_active_model();
    params.update(&mut user);
    user.updated_at = ActiveValue::Set(Local::now().naive_local());
    let user = user.update(&ctx.db).await?;
    format::json(CurrentResponse::new(&user))
}

#[utoipa::path(
    get,
    path = "/api/user/orders",
    tag = "users",
    responses(
        (status = 200, description = "Order list based on user login successfully", body = [OrderResponse]),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn user_order_list(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let orders = OrderEntity::find().filter(orders::Column::BuyerId.eq(user.id)).all(&ctx.db).await?;
    let response: Vec<OrderResponse> = orders
        .into_iter()
        .map(|o| OrderResponse {
            order_number: o.order_number,
            final_price: o.final_price,
            delivery_address_detail: None,
            payment_method_detail: o.payment_method_detail,
            status: o.status.to_value(),
            order_items: vec![],
        })
        .collect();
    format::json(response)
}

#[utoipa::path(
    post,
    path = "/api/user/update_profile",
    tag = "users",
    request_body = ProfileParams,
    responses(
        (status = 200, description = "Product update successfully", body = [CurrentResponse]),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
        (status = 404, description = "Product not found", body = UnauthorizedResponse),
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn update_profile(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<ProfileParams>,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    let mut user = user.into_active_model();
    params.update(&mut user);
    user.updated_at = ActiveValue::Set(Local::now().naive_local());
    let user = user.update(&ctx.db).await?;
    format::json(CurrentResponse::new(&user))
}

#[utoipa::path(
    post,
    path = "/api/user/{pid}/block",
    tag = "users",
    responses(
        (status = 200, description = "User successfully blocked"),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
    ),
    params(
        ("pid" = String, Path, description = "User database pid")
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn user_block(auth: auth::JWT, Path(pid): Path<String>, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;

    if user.pid == pid.parse::<Uuid>().unwrap() {
        return bad_request("cannot block yourself");
    } else if user.is_superuser {
        let u = users::Model::find_by_pid(&ctx.db, &pid).await?;
        u.into_active_model().set_blocked(&ctx.db).await?;
    }
    format::empty()
}

#[utoipa::path(
    post,
    path = "/api/user/{pid}/unblock",
    tag = "users",
    responses(
        (status = 200, description = "User successfully unblocked"),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
    ),
    params(
        ("pid" = String, Path, description = "User database pid")
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn user_unblock(auth: auth::JWT, Path(pid): Path<String>, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;

    if user.pid == pid.parse::<Uuid>().unwrap() {
        return bad_request("cannot block yourself");
    } else if user.is_superuser {
        let u = users::Model::find_by_pid(&ctx.db, &pid).await?;
        u.into_active_model().set_unblocked(&ctx.db).await?;
    }
    format::empty()
}

#[utoipa::path(
    delete,
    path = "/api/user/{pid}/delete",
    tag = "users",
    responses(
        (status = 200, description = "User successfully deleted"),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
    ),
    params(
        ("pid" = String, Path, description = "User database pid")
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn user_delete(auth: auth::JWT, Path(pid): Path<String>, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;

    if user.pid == pid.parse::<Uuid>().unwrap() {
        return bad_request("cannot delete yourself");
    } else if user.is_superuser {
        let u = users::Model::find_by_pid(&ctx.db, &pid.to_string()).await?;
        u.delete(&ctx.db).await?;
    }
    format::empty()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/user")
        .add("/current", get(current))
        .add("/update_location", post(update_location))
        .add("/update_profile", post(update_profile))
        .add("/products", get(product_list))
        .add("/orders", get(user_order_list))
        .add("/product/new", post(product_add))
        .add("/product/{id}", get(product_get_one))
        .add("/product/{id}", delete(product_remove))
        .add("/product/{id}", post(product_update))
        .add("/{id}/block", post(user_block))
        .add("/{id}/unblock", post(user_unblock))
        .add("/{id}/delete", delete(user_delete))
}
