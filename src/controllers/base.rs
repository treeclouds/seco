#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use crate::models::_entities::users;
use crate::views::base::BaseResponse;

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

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/")
        .add("/", get(hello))
        .add("/verify/{token}", get(verify_page))
}
