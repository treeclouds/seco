use seco::app::App;
use loco_rs::testing::prelude::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn can_request_root() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.get("/api/categories").await;
        assert_eq!(res.status_code(), 200);
    })
    .await;
}
