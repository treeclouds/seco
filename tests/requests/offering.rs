use seco::app::App;
use loco_rs::testing::prelude::*;
use serial_test::serial;

/// The offering endpoints are JWT-gated; a missing token must yield 401.
/// (Replaces the stale scaffold `echo`/`hello` tests that pointed at `/offering`
/// routes which no longer exist.)
#[tokio::test]
#[serial]
async fn offering_endpoints_require_auth() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.get("/api/offering/negotiations/fb-test").await;
        assert_eq!(res.status_code(), 401, "get negotiation must require auth");

        let res = request
            .post("/api/offering/negotiations/new")
            .json(&serde_json::json!({
                "product_id": 1,
                "offer": 100,
                "firebase_id": "fb-test"
            }))
            .await;
        assert_eq!(res.status_code(), 401, "add negotiation must require auth");

        let res = request
            .post("/api/offering/negotiations/1/do")
            .json(&serde_json::json!({ "status": "Accepted" }))
            .await;
        assert_eq!(res.status_code(), 401, "do negotiation must require auth");
    })
    .await;
}
