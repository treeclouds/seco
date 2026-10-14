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
            let auth_header = parts
                .headers
                .get(axum::http::header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");

            // jangan log token, cuma log “scheme”-nya
            let scheme = auth_header.split_whitespace().next().unwrap_or("");
            tracing::debug!(
                has_authorization = !auth_header.is_empty(),
                authorization_scheme = scheme,
                "OptionalJwt: inspecting Authorization header"
            );

            match auth::JWT::from_request_parts(parts, state).await {
                Ok(jwt) => Ok(Self(Some(jwt))),
                Err(err) => {
                    tracing::debug!(error = %err, "OptionalJwt: JWT invalid/missing, treating as guest");
                    Ok(Self(None))
                }
            }
        }
    }
}