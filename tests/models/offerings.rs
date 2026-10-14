use loco_rs::testing::prelude::*;
use sea_orm::prelude::Decimal;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, ModelTrait, Set};
use seco::app::App;
use seco::models::_entities::{offerings, products, users};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn offering_insert_find_update_delete() {
    let boot = boot_test::<App>().await.expect("boot test app");
    let db = &boot.app_context.db;

    let seller = users::ActiveModel {
        email: Set("of-seller@test.com".to_string()),
        password: Set("x".to_string()),
        first_name: Set("Seller".to_string()),
        last_name: Set("Model".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert seller");

    let buyer = users::ActiveModel {
        email: Set("of-buyer@test.com".to_string()),
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
        title: Set("OF product".to_string()),
        description: Set("d".to_string()),
        price: Set(Decimal::new(500, 0)),
        dimension_width: Set(1.0),
        dimension_height: Set(1.0),
        dimension_length: Set(1.0),
        dimension_weight: Set(1.0),
        stock: Set(5),
        sku: Set("SKU-OF".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert product");

    let offering = offerings::ActiveModel {
        product_id: Set(product.id),
        user_id: Set(buyer.id),
        product_name: Set("OF product".to_string()),
        product_price: Set(Decimal::new(500, 0)),
        offer_price: Set(Decimal::new(400, 0)),
        firebase_id: Set(Some("fb-of-1".to_string())),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert offering");

    assert_eq!(offering.user_id, buyer.id);
    assert_eq!(offering.offer_price, Decimal::new(400, 0));

    let found = offerings::Entity::find_by_id(offering.id)
        .one(db)
        .await
        .unwrap()
        .expect("find offering");
    assert_eq!(found.firebase_id.as_deref(), Some("fb-of-1"));

    let mut am = found.into_active_model();
    am.offer_price = Set(Decimal::new(450, 0));
    let updated = am.update(db).await.expect("update offering");
    assert_eq!(updated.offer_price, Decimal::new(450, 0));

    let res = updated.delete(db).await.expect("delete offering");
    assert_eq!(res.rows_affected, 1);
}
