use loco_rs::testing::prelude::*;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, ModelTrait, Set};
use seco::app::App;
use seco::models::_entities::categories;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn category_insert_find_update_delete() {
    let boot = boot_test::<App>().await.expect("boot test app");
    let db = &boot.app_context.db;

    let category = categories::ActiveModel {
        name: Set("Electronics".to_string()),
        parent_id: Set(None),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("insert category");

    assert_eq!(category.name, "Electronics");

    let found = categories::Entity::find_by_id(category.id)
        .one(db)
        .await
        .unwrap()
        .expect("find category");
    assert_eq!(found.name, "Electronics");

    let mut am = found.into_active_model();
    am.name = Set("Consumer Electronics".to_string());
    let updated = am.update(db).await.expect("update category");
    assert_eq!(updated.name, "Consumer Electronics");

    let res = updated.delete(db).await.expect("delete category");
    assert_eq!(res.rows_affected, 1);
}
