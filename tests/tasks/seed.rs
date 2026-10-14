use loco_rs::boot::run_task;
use loco_rs::task;
use loco_rs::testing::prelude::*;
use seco::app::App;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_can_seed_data() {
    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    assert!(run_task::<App>(
        &boot.app_context,
        Some(&"seed_data".to_string()),
        &task::Vars::default()
    )
        .await
        .is_ok());
}
