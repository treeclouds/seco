use seco::app::App;
use loco_rs::testing::prelude::*;
use serial_test::serial;
use serde_json::json;
use super::prepare_data::*;

/// Catalog mutations (brands/categories/materials): 401 anon, 403 user, 200 admin.
#[tokio::test]
#[serial]
async fn catalog_create_requires_superuser() {
    request::<App, _, _>(|request, ctx| async move {
        let user = create_user(&ctx.db, "cat-user@t.io", "user", false).await;
        let admin = create_user(&ctx.db, "cat-admin@t.io", "admin", true).await;
        let (uk, uv) = auth_header_for(&ctx, &user);
        let (ak, av) = auth_header_for(&ctx, &admin);
        let cases = [
            ("/api/brand/new", json!({"name": "B1", "code": null, "is_active": true})),
            ("/api/category/new", json!({"name": "C1", "parent_id": null, "is_active": true})),
            ("/api/material/new", json!({"name": "M1", "code": null, "is_active": true})),
        ];
        for (url, body) in cases {
            let res = request.post(url).json(&body).await;
            assert_eq!(res.status_code(), 401, "{url} anon");
            let res = request.post(url).add_header(uk.clone(), uv.clone()).json(&body).await;
            assert_eq!(res.status_code(), 403, "{url} non-admin");
            let res = request.post(url).add_header(ak.clone(), av.clone()).json(&body).await;
            assert_eq!(res.status_code(), 200, "{url} admin: {}", res.text());
        }
        // public reads stay public
        assert_eq!(request.get("/api/brands").await.status_code(), 200);
        assert_eq!(request.get("/api/materials").await.status_code(), 200);
    })
    .await;
}
