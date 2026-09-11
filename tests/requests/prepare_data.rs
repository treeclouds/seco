use axum::http::{HeaderName, HeaderValue};
use loco_rs::{app::AppContext, TestServer};
use sea_orm::{ActiveModelTrait, Set};
use seco::models::_entities::{categories, products};
use seco::{models::users, views::auth::LoginResponse};

const USER_EMAIL: &str = "test@loco.com";
const USER_PASSWORD: &str = "1234";

pub struct LoggedInUser {
    pub user: users::Model,
    pub token: String,
}

pub async fn init_user_login(request: &TestServer, ctx: &AppContext) -> LoggedInUser {
    let register_payload = serde_json::json!({
        "first_name": "loco",
        "last_name": "loco",
        "email": USER_EMAIL,
        "password": USER_PASSWORD
    });

    //Creating a new user
    request
        .post("/api/auth/register")
        .json(&register_payload)
        .await;
    let user = users::Model::find_by_email(&ctx.db, USER_EMAIL)
        .await
        .unwrap();

    let verify_payload = serde_json::json!({
        "token": user.email_verification_token,
    });

    request.post("/api/auth/verify").json(&verify_payload).await;

    let response = request
        .post("/api/auth/login")
        .json(&serde_json::json!({
            "email": USER_EMAIL,
            "password": USER_PASSWORD
        }))
        .await;

    let login_response: LoginResponse = serde_json::from_str(&response.text()).unwrap();

    LoggedInUser {
        user: users::Model::find_by_email(&ctx.db, USER_EMAIL)
            .await
            .unwrap(),
        token: login_response.token,
    }
}

pub fn auth_header(token: &str) -> (HeaderName, HeaderValue) {
    let auth_header_value = HeaderValue::from_str(&format!("Bearer {}", &token)).unwrap();

    (HeaderName::from_static("authorization"), auth_header_value)
}

/// Create a user directly in the DB (bypassing the register/verify/login HTTP
/// flow) and return the stored model. `pid` and `api_key` are auto-populated by
/// the model's `before_save` hook, so only identity + flag fields are required.
pub async fn create_user(
    db: &sea_orm::DatabaseConnection,
    email: &str,
    first_name: &str,
    superuser: bool,
) -> users::Model {
    users::ActiveModel {
        email: Set(email.to_string()),
        password: Set("hashed-not-used".to_string()),
        first_name: Set(first_name.to_string()),
        last_name: Set("Tester".to_string()),
        is_active: Set(true),
        is_superuser: Set(superuser),
        is_blocked: Set(false),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("failed to insert user")
}

/// Create a product owned by `seller_id` and return the stored model.
///
/// Seeds a category and binds `category_id` so the product survives the
/// `INNER JOIN categories` in `get_all_products` (a NULL `category_id` is
/// silently dropped by that query).
pub async fn create_product(
    db: &sea_orm::DatabaseConnection,
    seller_id: i32,
    title: &str,
    price: i64,
) -> products::Model {
    let category = categories::ActiveModel {
        name: Set(format!("Category-{title}")),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("failed to insert category");

    products::ActiveModel {
        seller_id: Set(seller_id),
        category_id: Set(Some(category.id)),
        title: Set(title.to_string()),
        description: Set("test product".to_string()),
        price: Set(sea_orm::prelude::Decimal::new(price, 0)),
        dimension_width: Set(1.0),
        dimension_height: Set(1.0),
        dimension_length: Set(1.0),
        dimension_weight: Set(1.0),
        stock: Set(10),
        sku: Set(format!("SKU-{title}")),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("failed to insert product")
}

/// Build a JWT auth header pair for a logged-in user, minting the token from
/// the model using the same secret the app validates against (read from the
/// test `AppContext` config, so it stays in sync with `config/test.yaml`).
pub fn auth_header_for(
    ctx: &loco_rs::app::AppContext,
    user: &users::Model,
) -> (HeaderName, HeaderValue) {
    let secret = ctx
        .config
        .get_jwt_config()
        .expect("jwt config")
        .secret
        .clone();
    let token = user.generate_jwt(&secret, 3600).expect("failed to mint jwt");
    let value = HeaderValue::from_str(&format!("Bearer {token}")).unwrap();
    (HeaderName::from_static("authorization"), value)
}
