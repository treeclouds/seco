use loco_rs::boot::{create_app, StartMode};
use loco_rs::environment::Environment;
use seco::app::App;
use migration::Migrator;
use shuttle_runtime::DeploymentMetadata;
use tracing::info;

#[shuttle_runtime::main]
async fn main(
  #[shuttle_shared_db::Postgres] conn_str: String,
  #[shuttle_runtime::Metadata] meta: DeploymentMetadata,
) -> shuttle_axum::ShuttleAxum {
    std::env::set_var("DATABASE_URL", conn_str);
    let environment = match meta.env {
        shuttle_runtime::Environment::Local => Environment::Development,
        shuttle_runtime::Environment::Deployment => Environment::Production,
    };
    info!("environment {:?}", environment);
    let boot_result = create_app::<App, Migrator>(StartMode::ServerOnly, &environment)
        .await
        .unwrap();

    let router = boot_result.router.unwrap();
    Ok(router.into())
}
