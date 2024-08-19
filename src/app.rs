use std::path::Path;

use async_trait::async_trait;
use axum::Router as AxumRouter;
use loco_rs::{
    app::{AppContext, Hooks},
    boot::{create_app, BootResult, StartMode},
    cache,
    controller::AppRoutes,
    db::{self, truncate_table},
    environment::Environment,
    storage::{self, Storage},
    task::Tasks,
    worker::{AppWorker, Processor},
    Result,
};
use migration::Migrator;
use sea_orm::DatabaseConnection;

use crate::{
    controllers::{
        self,
        auth::{self, VerifyParams, ResetParams, ForgotParams},
        products::{self as ct_products, ProductPostParams, UnauthorizedResponse},
        user::{self},
        upload::{self}
    },
    models::{
        users::{LoginParams, RegisterParams},
        _entities::{product_images, products, users}
    },
    tasks,
    views::{
        auth::LoginResponse,
        product::ProductResponse,
        user::CurrentResponse,
    },
    workers::downloader::DownloadWorker,
};

use utoipa::{
    openapi::security::{SecurityScheme, HttpBuilder, HttpAuthScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    info(title="Secondhand"),
    paths(
        auth::register,
        auth::verify,
        auth::forgot,
        auth::reset,
        auth::login,
        ct_products::list,
        ct_products::add,
        ct_products::update,
        ct_products::remove,
        ct_products::get_one,
        user::current,
        user::product_list,
        upload::upload_product_image_file,
    ),
    components(
        schemas(
            LoginParams, RegisterParams, VerifyParams, ResetParams, ForgotParams,
            ProductPostParams, LoginResponse, ProductResponse, UnauthorizedResponse,
            CurrentResponse
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "SecondHand", description = "SecondHand management API")
    )
)]
struct ApiDoc;
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "jwt_token",
                // SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("Authorization"))),
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                )
            )
        }
    }
}

pub struct App;
#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    fn app_version() -> String {
        format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA")
                .or(option_env!("GITHUB_SHA"))
                .unwrap_or("dev")
        )
    }

    async fn boot(mode: StartMode, environment: &Environment) -> Result<BootResult> {
        create_app::<Self, Migrator>(mode, environment).await
    }

    // async fn storage(
    //     _config: &Config,
    //     environment: &Environment,
    // ) -> Result<Option<storage::Storage>> {
    //     let store = if environment == &Environment::Test {
    //         storage::drivers::mem::new()
    //     } else {
    //         storage::drivers::local::new_with_prefix("storage-uploads").map_err(Box::from)?
    //     };
    //
    //     let storage = Storage::single(store);
    //     return Ok(Some(storage));
    // }

    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes()
            // .prefix("/api")
            .add_route(controllers::base::routes())
            .add_route(controllers::products::routes())
            .add_route(controllers::auth::routes())
            .add_route(controllers::user::routes())
            .add_route(controllers::upload::routes())
    }

    async fn after_routes(router: AxumRouter, _ctx: &AppContext) -> Result<AxumRouter> {
        let router_app = router
            .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()));
        Ok(router_app)
    }

    fn connect_workers<'a>(p: &'a mut Processor, ctx: &'a AppContext) {
        p.register(DownloadWorker::build(ctx));
    }

    fn register_tasks(tasks: &mut Tasks) {
        tasks.register(tasks::seed::SeedData);
    }

    async fn truncate(db: &DatabaseConnection) -> Result<()> {
        truncate_table(db, users::Entity).await?;
        truncate_table(db, products::Entity).await?;
        truncate_table(db, product_images::Entity).await?;
        Ok(())
    }

    async fn seed(db: &DatabaseConnection, base: &Path) -> Result<()> {
        db::seed::<users::ActiveModel>(db, &base.join("users.yaml").display().to_string()).await?;
        db::seed::<products::ActiveModel>(db, &base.join("products.yaml").display().to_string()).await?;
        db::seed::<product_images::ActiveModel>(db, &base.join("product_images.yaml").display().to_string()).await?;
        Ok(())
    }

    async fn after_context(ctx: AppContext) -> Result<AppContext> {
        let store = if ctx.environment == Environment::Test {
            storage::drivers::mem::new()
        } else {
            storage::drivers::local::new_with_prefix("storage-uploads").map_err(Box::from)?
        };

        Ok(AppContext {
            storage: Storage::single(store).into(),
            cache: cache::Cache::new(cache::drivers::inmem::new()).into(),
            ..ctx
        })
    }
}