use seco::app::App;
use loco_rs::testing::prelude::*;
use serial_test::serial;
use serde_json::json;
use super::prepare_data::*;

use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use seco::models::_entities::{orders, sea_orm_active_enums::OrderStatusEnum};

async fn place_order(request: &loco_rs::TestServer, ctx: &loco_rs::app::AppContext, tag: &str)
    -> (seco::models::users::Model, seco::models::users::Model, i32, orders::Model, i32)
{
    let seller = create_user(&ctx.db, &format!("o-seller-{tag}@t.io"), "seller", false).await;
    let buyer = create_user(&ctx.db, &format!("o-buyer-{tag}@t.io"), "buyer", false).await;
    let p = create_product(&ctx.db, seller.id, &format!("OrdProd{tag}"), 100).await;
    let pm = create_payment_method(&ctx.db).await;
    let addr = create_address(&ctx.db, buyer.id).await;
    let (k, v) = auth_header_for(ctx, &buyer);
    let res = request.post("/api/orders/new").add_header(k, v)
        .json(&json!({"final_price": 1, "payment_method_id": pm.id, "product_id": p.id, "delivery_address_id": addr.id}))
        .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
    let id = res.json::<serde_json::Value>()["id"].as_i64().unwrap() as i32;
    let order = orders::Entity::find_by_id(id).one(&ctx.db).await.unwrap().unwrap();
    (seller, buyer, p.id, order, pm.id)
}

#[tokio::test]
#[serial]
async fn cancel_restores_stock_once_and_confirm_rejects_cancelled() {
    request::<App, _, _>(|request, ctx| async move {
        let (_s, buyer, pid, order, _) = place_order(&request, &ctx, "a").await;
        assert_eq!(product_stock(&ctx.db, pid).await, 9);
        let (k, v) = auth_header_for(&ctx, &buyer);
        let cancel = format!("/api/user/order/{}/cancel", order.order_number);
        let confirm = format!("/api/orders/{}/confirm", order.id);

        assert_eq!(request.get(&cancel).add_header(k.clone(), v.clone()).await.status_code(), 405);
        assert_eq!(request.get(&confirm).add_header(k.clone(), v.clone()).await.status_code(), 405);

        let res = request.post(&cancel).add_header(k.clone(), v.clone()).await;
        assert_eq!(res.status_code(), 200, "{}", res.text());
        assert_eq!(product_stock(&ctx.db, pid).await, 10, "stock restored");

        let res = request.post(&cancel).add_header(k.clone(), v.clone()).await;
        assert_eq!(res.status_code(), 400, "second cancel rejected");
        assert_eq!(product_stock(&ctx.db, pid).await, 10, "no double restore");

        let res = request.post(&confirm).add_header(k.clone(), v.clone()).await;
        assert_eq!(res.status_code(), 400, "confirm cancelled order");
        let o = orders::Entity::find_by_id(order.id).one(&ctx.db).await.unwrap().unwrap();
        assert_eq!(o.status, OrderStatusEnum::Cancelled);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn seller_cancel_is_post_and_restores_stock() {
    request::<App, _, _>(|request, ctx| async move {
        let (seller, _b, pid, order, _) = place_order(&request, &ctx, "s").await;
        let (k, v) = auth_header_for(&ctx, &seller);
        let url = format!("/api/seller/order/{}/cancel", order.order_number);
        assert_eq!(request.get(&url).add_header(k.clone(), v.clone()).await.status_code(), 405);
        assert_eq!(request.post(&url).add_header(k, v).await.status_code(), 200);
        assert_eq!(product_stock(&ctx.db, pid).await, 10);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn confirm_shipped_order_ok() {
    request::<App, _, _>(|request, ctx| async move {
        let (_s, buyer, _pid, order, _) = place_order(&request, &ctx, "c").await;
        let mut am: orders::ActiveModel = order.clone().into();
        am.status = Set(OrderStatusEnum::Shipped);
        am.update(&ctx.db).await.unwrap();
        let (k, v) = auth_header_for(&ctx, &buyer);
        let url = format!("/api/orders/{}/confirm", order.id);
        assert_eq!(request.post(&url).add_header(k.clone(), v.clone()).await.status_code(), 200);
        assert_eq!(request.post(&url).add_header(k, v).await.status_code(), 400, "already completed");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn order_update_rejects_foreign_address_and_bad_payment_method() {
    request::<App, _, _>(|request, ctx| async move {
        let (seller, buyer, pid, order, pm) = place_order(&request, &ctx, "u").await;
        let foreign = create_address(&ctx.db, seller.id).await;
        let mine = create_address(&ctx.db, buyer.id).await;
        let (k, v) = auth_header_for(&ctx, &buyer);
        let url = format!("/api/orders/{}", order.id);

        let res = request.put(&url).add_header(k.clone(), v.clone())
            .json(&json!({"final_price": 1, "payment_method_id": pm, "product_id": pid, "delivery_address_id": foreign.id})).await;
        assert_eq!(res.status_code(), 400, "foreign address");
        let res = request.put(&url).add_header(k.clone(), v.clone())
            .json(&json!({"final_price": 1, "payment_method_id": 999_999, "product_id": pid, "delivery_address_id": mine.id})).await;
        assert_eq!(res.status_code(), 400, "bad payment method");

        let o = orders::Entity::find_by_id(order.id).one(&ctx.db).await.unwrap().unwrap();
        assert_ne!(o.delivery_address_id, Some(foreign.id));

        // another user cannot update it
        let (sk, sv) = auth_header_for(&ctx, &seller);
        let res = request.put(&url).add_header(sk, sv)
            .json(&json!({"final_price": 1, "payment_method_id": pm, "product_id": pid, "delivery_address_id": foreign.id})).await;
        assert_eq!(res.status_code(), 404);

        let res = request.put(&url).add_header(k, v)
            .json(&json!({"final_price": 1, "payment_method_id": pm, "product_id": pid, "delivery_address_id": mine.id})).await;
        assert_eq!(res.status_code(), 200, "{}", res.text());
    })
    .await;
}
