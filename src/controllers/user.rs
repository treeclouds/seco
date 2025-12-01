use chrono::Local;
use std::path::PathBuf;
use axum::extract::Multipart;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use bytes::Bytes;
use crate::{
    models::_entities::{
        users::{self, ActiveModel},
        products::{self, ActiveModel as ProductActiveModel, Entity as ProductEntity, Model as ProductModel},
        product_images::{ActiveModel as ProductImageActiveModel, Model as ProductImageModel}
    },
    views::user::CurrentResponse
};
use crate::controllers::{
    upload::generate_unique_filename,
    products::{ProductPostParams, UnauthorizedResponse}
};
use crate::views::product::ProductResponse;
use crate::views::product_image::ProductImageResponse;


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

struct PendingFile {
    file_name: String,
    content: Bytes,
}

#[derive(Serialize, Deserialize)]
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
    request_body = ProductPostParams,
    responses(
        (status = 200, description = "Create a new product successfully", body = ProductResponse)
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

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/user")
        .add("/current", get(current))
        .add("/update_location", post(update_location))
        .add("/products", get(product_list))
        .add("/product/new", post(product_add))
        .add("/product/{id}", get(product_get_one))
        .add("/product/{id}", delete(product_remove))
        .add("/product/{id}", post(product_update))
}
