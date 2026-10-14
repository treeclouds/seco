use seco::app::App;
use loco_rs::testing::prelude::*;
use serial_test::serial;
use serde_json::json;
use super::prepare_data::*;

#[tokio::test]
#[serial]
async fn cart_list_ok_and_qty_validated() {
    request::<App, _, _>(|request, ctx| async move {
        let seller = create_user(&ctx.db, "c-seller@t.io", "seller", false).await;
        let buyer = create_user(&ctx.db, "c-buyer@t.io", "buyer", false).await;
        let p = create_product(&ctx.db, seller.id, "CartProd", 100).await;
        let (k, v) = auth_header_for(&ctx, &buyer);

        for qty in [0, -2] {
            let res = request.post("/api/user/carts/new").add_header(k.clone(), v.clone())
                .json(&json!({"product_id": p.id, "qty": qty})).await;
            assert_eq!(res.status_code(), 400, "qty {qty}");
        }
        let res = request.post("/api/user/carts/new").add_header(k.clone(), v.clone())
            .json(&json!({"product_id": p.id, "qty": 1})).await;
        assert_eq!(res.status_code(), 200, "{}", res.text());
        let cart_id = res.json::<serde_json::Value>()["id"].as_i64().unwrap();

        // regression: FROM cart -> carts (was 500)
        let res = request.get("/api/user/carts").add_header(k.clone(), v.clone()).await;
        assert_eq!(res.status_code(), 200, "{}", res.text());
        assert_eq!(res.json::<serde_json::Value>().as_array().unwrap().len(), 1);

        // update to bad qty also rejected
        let res = request.put(&format!("/api/user/carts/{cart_id}")).add_header(k.clone(), v.clone())
            .json(&json!({"product_id": p.id, "qty": -1})).await;
        assert_eq!(res.status_code(), 400);

        // another user cannot touch it
        let (sk, sv) = auth_header_for(&ctx, &seller);
        let res = request.get(&format!("/api/user/carts/{cart_id}")).add_header(sk, sv).await;
        assert_eq!(res.status_code(), 404);
    })
    .await;
}
