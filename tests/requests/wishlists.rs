use seco::app::App;
use loco_rs::testing::prelude::*;
use serial_test::serial;

/// The wishlist endpoints are JWT-gated; a missing token must yield 401.
/// (Replaces the stale scaffold `echo`/`hello` tests that pointed at
/// `/wishlists` routes which no longer exist.)
#[tokio::test]
#[serial]
async fn wishlist_endpoints_require_auth() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.get("/api/user/wishlists").await;
        assert_eq!(res.status_code(), 401, "wishlist list must require auth");

        let res = request
            .post("/api/user/wishlists/new")
            .json(&serde_json::json!({ "product_id": 1 }))
            .await;
        assert_eq!(res.status_code(), 401, "wishlist add must require auth");

        let res = request.delete("/api/user/wishlists/1/remove").await;
        assert_eq!(res.status_code(), 401, "wishlist delete must require auth");
    })
    .await;
}
