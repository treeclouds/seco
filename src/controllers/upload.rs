#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]

use std::path::PathBuf;

use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, QueryFilter};
use axum::extract::Multipart;

use crate::views::product_image::ProductImageResponse;

use crate::models::_entities::{
    users, 
    products::{self, Entity, Model},
    product_images,
};

async fn load_product(ctx: &AppContext, user: users::Model, id: i32) -> Result<Model> {
    let item = user.find_related(Entity).filter(products::Column::Id.eq(id)).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

async fn generate_unique_filename(base_filename: &str) -> std::io::Result<PathBuf> {
    let mut filename = base_filename.to_owned();
    let mut counter = 0u32;

    loop {
        let path_buf = PathBuf::from("storage-uploads/product_images").join(&filename);
        if !path_buf.exists() {
            return Ok(filename.into());
        }

        let (name, ext) = base_filename.split_once('.').unwrap();

        counter += 1;
        filename = format!("{}_{:03}.{}", name, counter, ext.to_string());
    }
}

/// File upload example
///
/// ## Request Example
///
/// curl -H "Content-Type: multipart/form-data" -F "file=@./test-2.json"
/// 127.0.0.1:3000/upload/file
async fn upload_product_image_file(auth: auth::JWT, Path(product_id): Path<i32>, State(ctx): State<AppContext>, mut multipart: Multipart) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    load_product(&ctx, user, product_id).await?;
    let mut file = None;
    while let Some(field) = multipart.next_field().await.map_err(|err| {
        tracing::error!(error = ?err,"could not read multipart");
        Error::BadRequest("could not read multipart".into())
    })? {
        let file_name = match field.file_name() {
            Some(file_name) => file_name.to_string(),
            _ => return Err(Error::BadRequest("file name not found".into())),
        };

        let content = field.bytes().await.map_err(|err| {
            tracing::error!(error = ?err,"could not read bytes");
            Error::BadRequest("could not read bytes".into())
        })?;

        let unique_file_name = generate_unique_filename(&file_name).await?;

        let path = PathBuf::from("product_images").join(unique_file_name);
        ctx.storage.as_ref().upload(path.as_path(), &content).await?;

        file = Some(path);
    }

    let image_path = file.map_or_else( || PathBuf::from("default_path.txt"), |path| path);
    let product_image = product_images::ActiveModel {
        product_id: Set(product_id),
        image: Set(image_path.to_string_lossy().into_owned()),
        ..Default::default() // all other attributes are `NotSet`
    };
    
    let product_image: product_images::Model = product_image.insert(&ctx.db).await?;
    format::json(ProductImageResponse::new(&product_image))

}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/upload")
        .add("/:product_id/product_image_file", post(upload_product_image_file))
}