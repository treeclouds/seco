use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use loco_rs::prelude::*;
use std::convert::Infallible;

pub struct OptionalJwt(pub Option<auth::JWT>);

impl FromRequestParts<AppContext> for OptionalJwt {
    type Rejection = Infallible;

    fn from_request_parts(
        parts: &mut Parts,
        state: &AppContext,
    ) -> impl std::future::Future<Output = std::result::Result<Self, Self::Rejection>> + Send {
        async move {
            match auth::JWT::from_request_parts(parts, state).await {
                Ok(jwt) => Ok(Self(Some(jwt))),
                Err(_) => Ok(Self(None)),
            }
        }
    }
}