#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use crate::views::base::BaseResponse;

pub async fn echo(req_body: String) -> String {
    req_body
}

pub async fn hello(State(_ctx): State<AppContext>) -> Result<Response> {
    // do something with context (database, etc)
    let message = "Welcome to the Secondhand API";
    format::json(BaseResponse::new(&"ok".to_string(), &message.to_string()))
}

pub fn routes() -> Routes {
    Routes::new().add("/", get(hello))
}
