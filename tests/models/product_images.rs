use loco_rs::testing::prelude::*;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, ModelTrait, Set};
use seco::app::App;
use seco::models::_entities::{product_images, products, users};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn product_image_insert_find_update_delete() {
    let boot = boot_test::<App>().await.expect("boot test app");
    let db = &boot.app_context.db;

    let seller = users::ActiveModel {
        email: Set("pi-seller@test.com".to_string()),
        password: Set("x".to_string()),
        first_name: Set("Seller".to_string()),
        last_name: Set("Model".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert seller");

    let product = products::ActiveModel {
        seller_id: Set(seller.id),
        title: Set("PI product".to_string()),
        description: Set("d".to_string()),
        price: Set(sea_orm::prelude::Decimal::new(100, 0)),
        dimension_width: Set(1.0),
        dimension_height: Set(1.0),
        dimension_length: Set(1.0),
        dimension_weight: Set(1.0),
        stock: Set(5),
        sku: Set("SKU-PI".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert product");

    let image = product_images::ActiveModel {
        product_id: Set(product.id),
        image: Set("product_images/1/photo.jpg".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert product image");

    assert_eq!(image.product_id, product.id);

    let found = product_images::Entity::find_by_id(image.id)
        .one(db)
        .await
        .unwrap()
        .expect("find product image");
    assert_eq!(found.image, "product_images/1/photo.jpg");

    let mut am = found.into_active_model();
    am.image = Set("product_images/1/photo2.jpg".to_string());
    let updated = am.update(db).await.expect("update product image");
    assert_eq!(updated.image, "product_images/1/photo2.jpg");

    let res = updated.delete(db).await.expect("delete product image");
    assert_eq!(res.rows_affected, 1);
}
