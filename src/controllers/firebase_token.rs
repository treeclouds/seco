//! Mints Firebase Auth custom tokens so the mobile app can sign in to
//! Firebase as the backend user (uid = user pid). Firestore security rules
//! then key chat access on `request.auth.uid`.
//!
//! Configuration (one of):
//! - `FIREBASE_SERVICE_ACCOUNT`: the service-account JSON itself, or a path to it
//! - `FIREBASE_SERVICE_ACCOUNT_PATH`: path to the service-account JSON file
//!
//! When not configured the endpoint returns 503 and the rest of the API is unaffected.
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::users;

pub const FIREBASE_AUDIENCE: &str =
    "https://identitytoolkit.googleapis.com/google.identity.identitytoolkit.v1.IdentityToolkit";
/// Firebase rejects custom tokens that live longer than one hour.
const TOKEN_TTL_SECS: i64 = 3600;

#[derive(Debug, Deserialize)]
pub struct ServiceAccount {
    pub client_email: String,
    pub private_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomTokenClaims {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub iat: i64,
    pub exp: i64,
    pub uid: String,
}

#[derive(Debug, Serialize)]
pub struct FirebaseTokenResponse {
    pub token: String,
    pub uid: String,
    pub expires_in: i64,
}

/// Parse the service account from a raw env value (inline JSON or a file path).
pub fn parse_service_account(raw: &str) -> std::result::Result<ServiceAccount, String> {
    let trimmed = raw.trim();
    let json = if trimmed.starts_with('{') {
        trimmed.to_string()
    } else {
        std::fs::read_to_string(trimmed)
            .map_err(|e| format!("cannot read service account file: {e}"))?
    };
    let sa: ServiceAccount =
        serde_json::from_str(&json).map_err(|e| format!("invalid service account JSON: {e}"))?;
    if sa.client_email.is_empty() || sa.private_key.is_empty() {
        return Err("service account is missing client_email or private_key".into());
    }
    Ok(sa)
}

pub fn load_service_account() -> std::result::Result<ServiceAccount, String> {
    let raw = std::env::var("FIREBASE_SERVICE_ACCOUNT")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| {
            std::env::var("FIREBASE_SERVICE_ACCOUNT_PATH")
                .ok()
                .filter(|v| !v.trim().is_empty())
        })
        .ok_or_else(|| "Firebase service account is not configured".to_string())?;
    parse_service_account(&raw)
}

pub fn mint_custom_token(
    sa: &ServiceAccount,
    uid: &str,
    now: i64,
) -> std::result::Result<String, String> {
    let claims = CustomTokenClaims {
        iss: sa.client_email.clone(),
        sub: sa.client_email.clone(),
        aud: FIREBASE_AUDIENCE.to_string(),
        iat: now,
        exp: now + TOKEN_TTL_SECS,
        uid: uid.to_string(),
    };
    let key = EncodingKey::from_rsa_pem(sa.private_key.as_bytes())
        .map_err(|e| format!("invalid service account private key: {e}"))?;
    encode(&Header::new(Algorithm::RS256), &claims, &key)
        .map_err(|e| format!("failed to sign custom token: {e}"))
}

#[debug_handler]
async fn firebase_token(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    // Make sure the user still exists before handing out a Firebase identity.
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;

    let sa = match load_service_account() {
        Ok(sa) => sa,
        Err(e) => {
            tracing::warn!(error = %e, "firebase custom token requested but not configured");
            return format::render()
                .status(503)
                .json(serde_json::json!({"error": "firebase_not_configured", "description": "Chat sign-in is temporarily unavailable"}));
        }
    };

    let now = chrono::Utc::now().timestamp();
    match mint_custom_token(&sa, &user.pid.to_string(), now) {
        Ok(token) => format::json(FirebaseTokenResponse {
            token,
            uid: user.pid.to_string(),
            expires_in: TOKEN_TTL_SECS,
        }),
        Err(e) => {
            tracing::error!(error = %e, "failed to mint firebase custom token");
            format::render()
                .status(503)
                .json(serde_json::json!({"error": "firebase_not_configured", "description": "Chat sign-in is temporarily unavailable"}))
        }
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/auth")
        .add("/firebase-token", post(firebase_token))
}
