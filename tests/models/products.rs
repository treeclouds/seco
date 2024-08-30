use insta::assert_debug_snapshot;
use seco::{
    app::App,
    models::products::{self, Model},
};
use loco_rs::testing;
use sea_orm::{ActiveModelTrait, ActiveValue};
use serial_test::serial;
use sea_orm::prelude::Decimal;
use seco::models::users;

macro_rules! configure_insta {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        let _guard = settings.bind_to_scope();
    };
}

#[tokio::test]
#[serial]
async fn test_can_validate_product_model() {
    configure_insta!();

    let boot = testing::boot_test::<App>().await.unwrap();

    let user_res = users::ActiveModel {
        first_name: ActiveValue::set("User".to_string()),
        last_name: ActiveValue::set("Test".to_string()),
        email: ActiveValue::set("test@gmail.com".to_string()),
        password: ActiveValue::set("tester1234".to_string()),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await;

    let res = products::ActiveModel {
        seller_id: ActiveValue::set(user_res.unwrap().id),
        category: ActiveValue::set("Category test".to_string()),
        title: ActiveValue::set("Product 1".to_string()),
        description: ActiveValue::set("Product 1".to_string()),
        price: ActiveValue::set(Decimal::new(40000, 0)),
        dimension_width: ActiveValue::set(1f32),
        dimension_height: ActiveValue::set(5f32),
        dimension_length: ActiveValue::set(3f32),
        dimension_weight: ActiveValue::set(2f32),
        brand: ActiveValue::set("test brand".to_string()),
        material: ActiveValue::set("test material".to_string()),
        stock: ActiveValue::set(3),
        sku: ActiveValue::set("test sku".to_string()),
        ..Default::default()
    }
    .insert(&boot.app_context.db)
    .await;

    assert_debug_snapshot!(res);
}

#[tokio::test]
#[serial]
async fn test_get_product_by_id() {
    configure_insta!();

    let boot = testing::boot_test::<App>().await.unwrap();
    testing::seed::<App>(&boot.app_context.db).await.unwrap();

    // query your model, e.g.:
    //
    // let item = models::posts::Model::find_by_pid(
    //     &boot.app_context.db,
    //     "11111111-1111-1111-1111-111111111111",
    // )
    // .await;

    // snapshot the result:
    // assert_debug_snapshot!(item);
}
