#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]

use std::path::PathBuf;

use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, QueryFilter};
use axum::extract::Multipart;
use utoipa::ToSchema;
use serde::Deserialize;

use crate::views::product_image::ProductImageResponse;
use crate::controllers::products::UnauthorizedResponse;

use crate::models::_entities::{
    users, 
    products::{self, Entity, Model},
    product_images,
};

#[allow(dead_code)]
#[derive(ToSchema, Deserialize)]// Optional: gives a custom name to the schema
struct UploadedFileForm {
    /// Description for the file field in Swagger UI
    #[schema(content_media_type = "application/octet-stream", format = "binary")]
    file: String, // The actual type depends on your framework (e.g., Vec<u8>, Bytes, etc.)
}

async fn load_product(ctx: &AppContext, user: users::Model, id: i32) -> Result<Model> {
    let item = user.find_related(Entity).filter(products::Column::Id.eq(id)).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

const ALLOWED_IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif", "heic", "heif"];
const MAX_IMAGE_BYTES: usize = 10 * 1024 * 1024;

/// Reduce a client-supplied file name to a safe basename with an allowed image
/// extension. Rejects path separators, `..`, absolute paths, control chars and
/// unknown extensions. Returns (stem, lowercase extension).
pub fn sanitize_image_filename(raw: &str) -> std::result::Result<(String, String), &'static str> {
    if raw.contains('/') || raw.contains('\\') || raw.contains("..") || raw.contains('\0') {
        return Err("invalid file name");
    }
    let name = raw.trim();
    if name.is_empty() || name.starts_with('.') {
        return Err("invalid file name");
    }
    let (stem, ext) = match name.rsplit_once('.') {
        Some((s, e)) if !s.is_empty() && !e.is_empty() => (s, e.to_ascii_lowercase()),
        _ => return Err("file must have an image extension"),
    };
    if !ALLOWED_IMAGE_EXTENSIONS.contains(&ext.as_str()) {
        return Err("unsupported file type; allowed: jpg, jpeg, png, webp, gif, heic, heif");
    }
    let stem: String = stem
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .take(100)
        .collect();
    Ok((stem, ext))
}

/// Very small magic-byte sniff so a renamed non-image is rejected.
fn looks_like_image(ext: &str, b: &[u8]) -> bool {
    match ext {
        "jpg" | "jpeg" => b.starts_with(&[0xFF, 0xD8, 0xFF]),
        "png" => b.starts_with(&[0x89, b'P', b'N', b'G']),
        "gif" => b.starts_with(b"GIF8"),
        "webp" => b.len() >= 12 && &b[0..4] == b"RIFF" && &b[8..12] == b"WEBP",
        "heic" | "heif" => b.len() >= 12 && &b[4..8] == b"ftyp",
        _ => false,
    }
}

pub async fn generate_unique_filename(stem: &str, ext: &str, product_id: i32) -> std::io::Result<PathBuf> {
    let mut filename = format!("{stem}.{ext}");
    let mut counter = 0u32;

    loop {
        let path_buf = PathBuf::from("storage-uploads/product_images")
            .join(product_id.to_string())
            .join(&filename);
        if !path_buf.exists() {
            return Ok(filename.into());
        }
        counter += 1;
        filename = format!("{stem}_{counter:03}.{ext}");
    }
}

/// File upload example
///
/// ## Request Example
///
/// curl -H "Content-Type: multipart/form-data" -F "file=@./test-2.json"
/// 127.0.0.1:3000/api/upload/{product_id}/product_image_file
#[utoipa::path(
    post,
    path = "/api/upload/{product_id}/product_image_file",
    tag = "uploads",
    request_body(content = UploadedFileForm, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Product list based on user login successfully", body = [ProductImageResponse]),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
    ),
    params(
        ("product_id" = i32, Path, description = "Product database id")
    ),
    security(
        ("jwt_token" = [])
    )
)]
async fn upload_product_image_file(
    auth: auth::JWT,
    Path(product_id): Path<i32>,
    State(ctx): State<AppContext>,
    mut multipart: Multipart,
) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    load_product(&ctx, user, product_id).await?;

    let mut images: Vec<ProductImageResponse> = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| {
            tracing::error!(error = ?err,"could not read multipart");
            Error::BadRequest("could not read multipart".into())
        })?
    {
        let file_name = match field.file_name() {
            Some(file_name) => file_name.to_string(),
            _ => return Err(Error::BadRequest("file name not found".into())),
        };

        let (stem, ext) = sanitize_image_filename(&file_name)
            .map_err(|e| Error::BadRequest(e.into()))?;

        let content = field.bytes().await.map_err(|err| {
            tracing::error!(error = ?err,"could not read bytes");
            Error::BadRequest("could not read bytes".into())
        })?;

        if content.len() > MAX_IMAGE_BYTES {
            return Err(Error::BadRequest("file too large".into()));
        }
        if !looks_like_image(&ext, &content) {
            return Err(Error::BadRequest("file content is not a valid image".into()));
        }

        // generate nama file unik per product
        let unique_file_name = generate_unique_filename(&stem, &ext, product_id).await?;
        let unique_file_name_str = unique_file_name.to_string_lossy().to_string();
        let new_filename = format!("{product_id}/{unique_file_name_str}");

        let path = PathBuf::from("product_images").join(&new_filename);
        // defense in depth: final path must be exactly product_images/<id>/<basename>
        if path.components().count() != 3
            || path.components().any(|c| !matches!(c, std::path::Component::Normal(_)))
        {
            return Err(Error::BadRequest("invalid file name".into()));
        }

        // upload ke storage
        ctx.storage
            .as_ref()
            .upload(path.as_path(), &content)
            .await?;

        // simpan record ke DB
        let product_image = product_images::ActiveModel {
            product_id: Set(product_id),
            image: Set(path.to_string_lossy().into_owned()),
            ..Default::default()
        };

        let product_image: product_images::Model = product_image.insert(&ctx.db).await?;
        images.push(ProductImageResponse::new(&product_image));
    }

    if images.is_empty() {
        return bad_request("no files uploaded")
    }

    // sekarang return list image
    format::json(images)
}


pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/upload")
        .add("/{product_id}/product_image_file", post(upload_product_image_file))
}