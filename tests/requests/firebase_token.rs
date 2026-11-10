use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use loco_rs::testing::prelude::*;
use seco::{
    app::App,
    controllers::firebase_token::{
        mint_custom_token, parse_service_account, CustomTokenClaims, FIREBASE_AUDIENCE,
    },
};
use serial_test::serial;

use super::prepare_data::*;

/// Generates a throwaway RSA keypair with openssl (test-only, never committed).
fn test_keypair() -> (String, String) {
    let dir = std::env::temp_dir().join(format!("fbkey-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let priv_path = dir.join("k.pem");
    let pub_path = dir.join("k.pub.pem");
    let ok = std::process::Command::new("openssl")
        .args([
            "genpkey",
            "-algorithm",
            "RSA",
            "-pkeyopt",
            "rsa_keygen_bits:2048",
            "-out",
        ])
        .arg(&priv_path)
        .status()
        .expect("openssl is required for this test")
        .success();
    assert!(ok);
    let ok = std::process::Command::new("openssl")
        .args(["pkey", "-pubout", "-in"])
        .arg(&priv_path)
        .arg("-out")
        .arg(&pub_path)
        .status()
        .unwrap()
        .success();
    assert!(ok);
    (
        std::fs::read_to_string(priv_path).unwrap(),
        std::fs::read_to_string(pub_path).unwrap(),
    )
}

fn decode_claims(token: &str, public_pem: &str) -> CustomTokenClaims {
    let mut v = Validation::new(Algorithm::RS256);
    v.set_audience(&[FIREBASE_AUDIENCE]);
    decode::<CustomTokenClaims>(
        token,
        &DecodingKey::from_rsa_pem(public_pem.as_bytes()).unwrap(),
        &v,
    )
    .unwrap()
    .claims
}

#[test]
fn service_account_parsing_rejects_bad_input() {
    assert!(parse_service_account("{not json").is_err());
    assert!(parse_service_account("/definitely/missing/file.json").is_err());
    assert!(parse_service_account(r#"{"client_email":"","private_key":""}"#).is_err());
    let sa = parse_service_account(
        r#"{"client_email":"a@b.iam.gserviceaccount.com","private_key":"x"}"#,
    )
    .unwrap();
    assert!(
        mint_custom_token(&sa, "uid", 0).is_err(),
        "garbage key must not sign"
    );
}

#[test]
fn mints_valid_firebase_custom_token() {
    let (priv_pem, pub_pem) = test_keypair();
    let sa_json = serde_json::json!({"client_email": "svc@proj.iam.gserviceaccount.com", "private_key": priv_pem}).to_string();
    let sa = parse_service_account(&sa_json).unwrap();
    let now = chrono::Utc::now().timestamp();
    let token = mint_custom_token(&sa, "user-pid-123", now).unwrap();
    let c = decode_claims(&token, &pub_pem);
    assert_eq!(c.uid, "user-pid-123");
    assert_eq!(c.iss, "svc@proj.iam.gserviceaccount.com");
    assert_eq!(c.sub, c.iss);
    assert_eq!(c.aud, FIREBASE_AUDIENCE);
    assert_eq!(c.exp - c.iat, 3600);
}

#[tokio::test]
#[serial]
async fn firebase_token_endpoint() {
    let (priv_pem, pub_pem) = test_keypair();
    request::<App, _, _>(|request, ctx| async move {
        let res = request.post("/api/auth/firebase-token").await;
        assert_eq!(res.status_code(), 401, "requires login");

        let user = create_user(&ctx.db, "fb-user@t.io", "fbuser", false).await;
        let (k, v) = auth_header_for(&ctx, &user);

        std::env::remove_var("FIREBASE_SERVICE_ACCOUNT");
        std::env::remove_var("FIREBASE_SERVICE_ACCOUNT_PATH");
        let res = request.post("/api/auth/firebase-token").add_header(k.clone(), v.clone()).await;
        assert_eq!(res.status_code(), 503, "unconfigured fails gracefully");

        let sa = serde_json::json!({"client_email": "svc@proj.iam.gserviceaccount.com", "private_key": priv_pem}).to_string();
        std::env::set_var("FIREBASE_SERVICE_ACCOUNT", sa);
        let res = request.post("/api/auth/firebase-token").add_header(k, v).await;
        std::env::remove_var("FIREBASE_SERVICE_ACCOUNT");
        assert_eq!(res.status_code(), 200, "{}", res.text());
        let body: serde_json::Value = res.json();
        assert_eq!(body["uid"], user.pid.to_string());
        let c = decode_claims(body["token"].as_str().unwrap(), &pub_pem);
        assert_eq!(c.uid, user.pid.to_string());
    })
    .await;
}
