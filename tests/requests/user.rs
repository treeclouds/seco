use insta::{assert_debug_snapshot, with_settings};
use loco_rs::testing::prelude::*;
use seco::app::App;
use serial_test::serial;

use super::prepare_data;

// TODO: see how to dedup / extract this to app-local test utils
// not to framework, because that would require a runtime dep on insta
macro_rules! configure_insta {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("user_request");
        let _guard = settings.bind_to_scope();
    };
}

#[tokio::test]
#[serial]
async fn can_get_current_user() {
    configure_insta!();

    request::<App, _, _>(|request, ctx| async move {
        let user = prepare_data::init_user_login(&request, &ctx).await;

        let (auth_key, auth_value) = prepare_data::auth_header(&user.token);
        let response = request
            .get("/api/user/current")
            .add_header(auth_key, auth_value)
            .await;

        with_settings!({
            filters => cleanup_user_model()
        }, {
            assert_debug_snapshot!((response.status_code(), response.text()));
        });
    })
    .await;
}

// ---- Admin user management (superuser only) ----

#[tokio::test]
#[serial]
async fn can_list_users_as_superuser() {
    request::<App, _, _>(|request, ctx| async move {
        let admin = prepare_data::create_user(&ctx.db, "admin@test.com", "Admin", true).await;
        let (key, value) = prepare_data::auth_header_for(&ctx, &admin);

        let response = request.get("/api/users").add_header(key, value).await;

        assert_eq!(response.status_code(), 200);
        let body: serde_json::Value = serde_json::from_str(&response.text()).unwrap();
        let arr = body.as_array().expect("list should be an array");
        assert!(arr.iter().any(|u| u["email"].as_str() == Some("admin@test.com")));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn cannot_list_users_as_non_superuser() {
    request::<App, _, _>(|request, ctx| async move {
        let regular =
            prepare_data::create_user(&ctx.db, "regular@test.com", "Regular", false).await;
        let (key, value) = prepare_data::auth_header_for(&ctx, &regular);

        let response = request.get("/api/users").add_header(key, value).await;

        assert_eq!(response.status_code(), 401);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_create_user_as_superuser() {
    request::<App, _, _>(|request, ctx| async move {
        let admin = prepare_data::create_user(&ctx.db, "admin@test.com", "Admin", true).await;
        let (key, value) = prepare_data::auth_header_for(&ctx, &admin);

        let payload = serde_json::json!({
            "email": "newuser@test.com",
            "password": "Password123!",
            "first_name": "New",
            "last_name": "User",
            "phone": "08123456789",
            "is_active": true,
            "is_superuser": false
        });

        let response = request
            .post("/api/user/new")
            .add_header(key, value)
            .json(&payload)
            .await;

        assert_eq!(response.status_code(), 200);
        let body: serde_json::Value = serde_json::from_str(&response.text()).unwrap();
        assert_eq!(body["email"].as_str(), Some("newuser@test.com"));
        assert_eq!(body["first_name"].as_str(), Some("New"));
        assert_eq!(body["phone"].as_str(), Some("08123456789"));
        assert_eq!(body["is_active"].as_bool(), Some(true));
        assert_eq!(body["is_superuser"].as_bool(), Some(false));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn cannot_create_user_as_non_superuser() {
    request::<App, _, _>(|request, ctx| async move {
        let regular =
            prepare_data::create_user(&ctx.db, "regular@test.com", "Regular", false).await;
        let (key, value) = prepare_data::auth_header_for(&ctx, &regular);

        let payload = serde_json::json!({
            "email": "hacker@test.com",
            "password": "Password123!",
            "first_name": "Hacker",
            "last_name": "User"
        });

        let response = request
            .post("/api/user/new")
            .add_header(key, value)
            .json(&payload)
            .await;

        assert_eq!(response.status_code(), 401);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_get_user_as_superuser() {
    request::<App, _, _>(|request, ctx| async move {
        let admin = prepare_data::create_user(&ctx.db, "admin@test.com", "Admin", true).await;
        let target = prepare_data::create_user(&ctx.db, "getme@test.com", "Getme", false).await;
        let (key, value) = prepare_data::auth_header_for(&ctx, &admin);

        let response = request
            .get(&format!("/api/user/{}", target.pid))
            .add_header(key, value)
            .await;

        assert_eq!(response.status_code(), 200);
        let body: serde_json::Value = serde_json::from_str(&response.text()).unwrap();
        assert_eq!(body["email"].as_str(), Some("getme@test.com"));
        assert_eq!(body["first_name"].as_str(), Some("Getme"));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_update_user_as_superuser() {
    request::<App, _, _>(|request, ctx| async move {
        let admin = prepare_data::create_user(&ctx.db, "admin@test.com", "Admin", true).await;
        let target = prepare_data::create_user(&ctx.db, "target@test.com", "Target", false).await;
        let (key, value) = prepare_data::auth_header_for(&ctx, &admin);

        let payload = serde_json::json!({
            "first_name": "Updated",
            "phone": "0800"
        });

        let response = request
            .put(&format!("/api/user/{}", target.pid))
            .add_header(key, value)
            .json(&payload)
            .await;

        assert_eq!(response.status_code(), 200);
        let body: serde_json::Value = serde_json::from_str(&response.text()).unwrap();
        assert_eq!(body["first_name"].as_str(), Some("Updated"));
        assert_eq!(body["phone"].as_str(), Some("0800"));
        // untouched fields are preserved
        assert_eq!(body["last_name"].as_str(), Some("Tester"));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn cannot_update_user_as_non_superuser() {
    request::<App, _, _>(|request, ctx| async move {
        let regular =
            prepare_data::create_user(&ctx.db, "regular@test.com", "Regular", false).await;
        let target = prepare_data::create_user(&ctx.db, "victim@test.com", "Victim", false).await;
        let (key, value) = prepare_data::auth_header_for(&ctx, &regular);

        let response = request
            .put(&format!("/api/user/{}", target.pid))
            .add_header(key, value)
            .json(&serde_json::json!({"first_name": "Hacked"}))
            .await;

        assert_eq!(response.status_code(), 401);
    })
    .await;
}
