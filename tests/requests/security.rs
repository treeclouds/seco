use seco::app::App;
use loco_rs::testing::prelude::*;
use serial_test::serial;
use serde_json::json;
use super::prepare_data::*;

use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use seco::models::_entities::{products, sea_orm_active_enums::ProductStatus, users};

fn offer(pid: i32, amount: i64, fid: &str, action: &str) -> serde_json::Value {
    json!({"product_id": pid, "offer": amount, "firebase_id": fid, "action": action})
}

#[tokio::test]
#[serial]
async fn offers_validated_and_only_seller_accepts() {
    request::<App, _, _>(|request, ctx| async move {
        let seller = create_user(&ctx.db, "of-seller@t.io", "seller", false).await;
        let buyer = create_user(&ctx.db, "of-buyer@t.io", "buyer", false).await;
        let other = create_user(&ctx.db, "of-other@t.io", "other", false).await;
        let p = create_product(&ctx.db, seller.id, "OfferProd", 100).await;
        let (bk, bv) = auth_header_for(&ctx, &buyer);
        let (sk, sv) = auth_header_for(&ctx, &seller);
        let (ok, ov) = auth_header_for(&ctx, &other);
        let new = "/api/offering/negotiations/new";

        for amt in [-10, 0] {
            let res = request.post(new).add_header(bk.clone(), bv.clone()).json(&offer(p.id, amt, "fb-1", "Offer")).await;
            assert_eq!(res.status_code(), 400, "offer {amt}");
        }
        let res = request.post(new).add_header(sk.clone(), sv.clone()).json(&offer(p.id, 50, "fb-own", "Offer")).await;
        assert_eq!(res.status_code(), 400, "seller offering on own product");

        let res = request.post(new).add_header(bk.clone(), bv.clone()).json(&offer(p.id, 80, "fb-1", "Offer")).await;
        assert_eq!(res.status_code(), 200, "{}", res.text());
        let id = res.json::<serde_json::Value>()["id"].as_i64().unwrap();

        let res = request.post(new).add_header(bk.clone(), bv.clone()).json(&offer(p.id, 70, "fb-1", "Offer")).await;
        assert_eq!(res.status_code(), 409, "duplicate open firebase_id");
        let res = request.post(new).add_header(ok.clone(), ov.clone()).json(&offer(p.id, 70, "fb-1", "Offer")).await;
        assert_eq!(res.status_code(), 409, "hijack firebase_id");

        let do_url = format!("/api/offering/negotiations/{id}/do");
        let accept = json!({"status": "Accepted"});
        let res = request.post(&do_url).add_header(bk.clone(), bv.clone()).json(&accept).await;
        assert_eq!(res.status_code(), 403, "buyer accepts own offer");
        let res = request.post(&do_url).add_header(ok.clone(), ov.clone()).json(&accept).await;
        assert_eq!(res.status_code(), 403, "third party accepts");
        let res = request.get("/api/offering/negotiations/fb-1").add_header(ok, ov).await;
        assert_eq!(res.status_code(), 404, "third party reads negotiation");

        let res = request.post(&do_url).add_header(sk.clone(), sv.clone()).json(&accept).await;
        assert_eq!(res.status_code(), 200, "{}", res.text());
        let res = request.post(&do_url).add_header(sk, sv).json(&json!({"status": "Declined"})).await;
        assert_eq!(res.status_code(), 400, "closed offer can't change");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn product_update_rejects_negative_price_and_stock() {
    request::<App, _, _>(|request, ctx| async move {
        let seller = create_user(&ctx.db, "v-seller@t.io", "seller", false).await;
        let p = create_product(&ctx.db, seller.id, "ValProd", 100).await;
        let (k, v) = auth_header_for(&ctx, &seller);
        let base = |price: i64, stock: i64| json!({
            "title": "ValProd", "description": "d", "price": price, "stock": stock, "sku": "x",
            "dimension_width": 1, "dimension_height": 1, "dimension_length": 1, "dimension_weight": 1,
            "status": "Active", "category_id": p.category_id
        });
        let url = format!("/api/user/product/{}", p.id);
        for (price, stock) in [(-5, 1), (0, 1), (10, -3)] {
            let res = request.post(&url).add_header(k.clone(), v.clone()).json(&base(price, stock)).await;
            assert_eq!(res.status_code(), 400, "price {price} stock {stock}");
        }
        let res = request.post(&url).add_header(k, v).json(&base(10, 0)).await;
        assert_eq!(res.status_code(), 200, "{}", res.text());
        let stored = products::Entity::find_by_id(p.id).one(&ctx.db).await.unwrap().unwrap();
        assert_eq!(stored.stock, 0);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn drafts_hidden_publicly_and_condition_filter_works() {
    request::<App, _, _>(|request, ctx| async move {
        let seller = create_user(&ctx.db, "d-seller@t.io", "seller", false).await;
        let live = create_product(&ctx.db, seller.id, "LiveProd", 100).await;
        let draft = create_product(&ctx.db, seller.id, "DraftSecret", 100).await;
        let mut am: products::ActiveModel = draft.clone().into();
        am.status = Set(ProductStatus::Draft);
        am.update(&ctx.db).await.unwrap();
        let mut am: products::ActiveModel = live.clone().into();
        am.condition = Set(Some(seco::models::_entities::sea_orm_active_enums::Condition::Good));
        am.update(&ctx.db).await.unwrap();

        let res = request.get("/api/products").await;
        assert_eq!(res.status_code(), 200);
        assert!(res.text().contains("LiveProd"));
        assert!(!res.text().contains("DraftSecret"), "draft in public list");
        assert_eq!(request.get(&format!("/api/product/{}", draft.id)).await.status_code(), 404);
        let (k, v) = auth_header_for(&ctx, &seller);
        let res = request.get(&format!("/api/product/{}", draft.id)).add_header(k, v).await;
        assert_eq!(res.status_code(), 200, "seller still sees own draft");

        let res = request.get("/api/products?condition=Good").await;
        assert_eq!(res.status_code(), 200, "{}", res.text());
        let list = res.json::<serde_json::Value>();
        let titles: Vec<_> = list.as_array().unwrap().iter().map(|p| p["title"].as_str().unwrap().to_string()).collect();
        assert_eq!(titles, vec!["LiveProd".to_string()]);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn email_change_clears_verification_and_rejects_duplicates() {
    request::<App, _, _>(|request, ctx| async move {
        let taken = create_user(&ctx.db, "em-taken@t.io", "taken", false).await;
        let user = create_user(&ctx.db, "em-user@t.io", "user", false).await;
        let mut am: users::ActiveModel = user.clone().into();
        am.email_verified_at = Set(Some(chrono::Local::now().naive_local()));
        let user = am.update(&ctx.db).await.unwrap();
        let (k, v) = auth_header_for(&ctx, &user);
        let body = |email: &str| json!({"first_name": "userx", "last_name": "Tester", "email": email, "location": "c"});

        let res = request.post("/api/user/update_profile").add_header(k.clone(), v.clone()).json(&body(&taken.email)).await;
        assert_eq!(res.status_code(), 409, "{}", res.text());

        // same email: stays verified
        let res = request.post("/api/user/update_profile").add_header(k.clone(), v.clone()).json(&body("em-user@t.io")).await;
        assert_eq!(res.status_code(), 200);
        let u = users::Entity::find_by_id(user.id).one(&ctx.db).await.unwrap().unwrap();
        assert!(u.email_verified_at.is_some());

        let res = request.post("/api/user/update_profile").add_header(k, v).json(&body("em-new@t.io")).await;
        assert_eq!(res.status_code(), 200, "{}", res.text());
        let u = users::Entity::find_by_id(user.id).one(&ctx.db).await.unwrap().unwrap();
        assert_eq!(u.email, "em-new@t.io");
        assert!(u.email_verified_at.is_none(), "verification must be cleared");
        assert!(u.email_verification_token.is_some());
    })
    .await;
}
