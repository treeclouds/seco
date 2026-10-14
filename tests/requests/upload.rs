use seco::app::App;
use loco_rs::testing::prelude::*;
use serial_test::serial;
use serde_json::json;
use super::prepare_data::*;

use axum_test::multipart::{MultipartForm, Part};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use seco::models::_entities::product_images;

const PNG: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0];

fn form(name: &str, bytes: &[u8], mime: &str) -> MultipartForm {
    MultipartForm::new().add_part("file", Part::bytes(bytes.to_vec()).file_name(name).mime_type(mime))
}

#[tokio::test]
#[serial]
async fn upload_rejects_traversal_and_non_images_accepts_png() {
    request::<App, _, _>(|request, ctx| async move {
        let seller = create_user(&ctx.db, "up-seller@t.io", "seller", false).await;
        let p = create_product(&ctx.db, seller.id, "UpProd", 100).await;
        let (k, v) = auth_header_for(&ctx, &seller);
        let url = format!("/api/upload/{}/product_image_file", p.id);

        for bad in ["../../../../tmp/pwn.png", "..\\pwn.png", "/etc/pwn.png", ".hidden.png"] {
            let res = request.post(&url).add_header(k.clone(), v.clone())
                .multipart(form(bad, PNG, "image/png")).await;
            assert_eq!(res.status_code(), 400, "traversal name {bad} must be rejected");
        }
        // non-image extension, extensionless, and png extension with non-png bytes
        for (name, bytes) in [("evil.html", b"<script>".as_slice()), ("noext", PNG), ("fake.png", b"hello".as_slice())] {
            let res = request.post(&url).add_header(k.clone(), v.clone())
                .multipart(form(name, bytes, "image/png")).await;
            assert_eq!(res.status_code(), 400, "{name} must be rejected");
        }
        // nothing stored for the rejected uploads
        let stored = product_images::Entity::find()
            .filter(product_images::Column::ProductId.eq(p.id)).all(&ctx.db).await.unwrap();
        assert!(stored.is_empty(), "rejected uploads must not persist: {stored:?}");

        let res = request.post(&url).add_header(k.clone(), v.clone())
            .multipart(form("photo.png", PNG, "image/png")).await;
        assert_eq!(res.status_code(), 200, "{}", res.text());
        let stored = product_images::Entity::find()
            .filter(product_images::Column::ProductId.eq(p.id)).all(&ctx.db).await.unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].image, format!("product_images/{}/photo.png", p.id));
        assert!(!stored[0].image.contains(".."));
    })
    .await;
}
