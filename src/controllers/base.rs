#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use crate::models::_entities::users;
use crate::views::base::BaseResponse;
use crate::views;
use loco_rs::{auth::jwt};
use axum::{http::{StatusCode, HeaderMap}, middleware, response::{Redirect}};

pub async fn echo(req_body: String) -> String {
    req_body
}

pub async fn hello(State(_ctx): State<AppContext>) -> Result<Response> {
    // do something with context (database, etc.)
    let message = "Welcome to the Secondhand API";
    format::json(BaseResponse::new(&"ok".to_string(), &message.to_string()))
}

#[debug_handler]
pub async fn verify_page(
    State(ctx): State<AppContext>,
    Path(token): Path<String>,
) -> Result<Response> {
    tracing::info!("verify token from path: {}", token);

    let user = users::Model::find_by_verification_token(&ctx.db, &*token).await?;

    if user.email_verified_at.is_some() {
        return format::text("Account already verified!")
    } else {
        let active_model = user.into_active_model();
        let user = active_model.verified(&ctx.db).await?;
        tracing::info!(pid = user.pid.to_string(), "user verified");
    }

    format::text("Account has been verified successfully!")
}

pub async fn render_home(State(ctx): State<AppContext>, ViewEngine(v): ViewEngine<TeraView>) -> Result<impl IntoResponse> {
    views::base::home(v)
}

pub async fn render_dashboard(ViewEngine(v): ViewEngine<TeraView>) -> Result<impl IntoResponse> {
    views::base::dashboard(v)
}

pub fn routes() -> Routes {

    let dashboard_routes = Routes::new()
        .add("/", get(render_dashboard));
        // .add("/brands", get(list_brands))
        // .add("/materials", get(list_materials))
        // .layer(middleware::from_fn(auth_middleware));

    Routes::new()
        .prefix("/")
        .add("/", get(render_home))
        .nest("/dashboard", dashboard_routes)
        // .add("/dashboard", get(render_dashboard).layer(middleware::from_fn(auth_middleware)))
        .add("/verify/{token}", get(verify_page))
}
