use loco_rs::testing::prelude::*;
use sea_orm::prelude::Decimal;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, ModelTrait, Set};
use seco::app::App;
use seco::models::_entities::{products, users, wishlists};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn wishlist_insert_find_update_delete() {
    let boot = boot_test::<App>().await.expect("boot test app");
    let db = &boot.app_context.db;

    let seller = users::ActiveModel {
        email: Set("wl-seller@test.com".to_string()),
        password: Set("x".to_string()),
        first_name: Set("Seller".to_string()),
        last_name: Set("Model".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert seller");

    let buyer = users::ActiveModel {
        email: Set("wl-buyer@test.com".to_string()),
        password: Set("x".to_string()),
        first_name: Set("Buyer".to_string()),
        last_name: Set("Model".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert buyer");

    let product = products::ActiveModel {
        seller_id: Set(seller.id),
        title: Set("WL product".to_string()),
        description: Set("d".to_string()),
        price: Set(Decimal::new(100, 0)),
        dimension_width: Set(1.0),
        dimension_height: Set(1.0),
        dimension_length: Set(1.0),
        dimension_weight: Set(1.0),
        stock: Set(5),
        sku: Set("SKU-WL".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert product");

    let wishlist = wishlists::ActiveModel {
        user_id: Set(buyer.id),
        product_id: Set(product.id),
        is_deleted: Set(false),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert wishlist");

    assert_eq!(wishlist.user_id, buyer.id);
    assert!(!wishlist.is_deleted);

    let found = wishlists::Entity::find_by_id(wishlist.id)
        .one(db)
        .await
        .unwrap()
        .expect("find wishlist");
    assert_eq!(found.product_id, product.id);

    let mut am = found.into_active_model();
    am.is_deleted = Set(true);
    let updated = am.update(db).await.expect("update wishlist");
    assert!(updated.is_deleted);
}
