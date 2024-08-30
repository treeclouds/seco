use eyre::Context;
#[allow(unused_imports)]
use loco_rs::{cli::playground, prelude::*};
use sea_orm::{FromQueryResult, JoinType, RelationTrait, QuerySelect};
use uuid::Uuid;
use seco::app::App;
use seco::models::products::Entity as Products;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let ctx = playground::<App>().await.context("playground")?;

    // let active_model: articles::ActiveModel = ActiveModel {
    //     title: Set(Some("how to build apps in 3 steps".to_string())),
    //     content: Set(Some("use Loco: https://loco.rs".to_string())),
    //     ..Default::default()
    // };
    // active_model.insert(&ctx.db).await.unwrap();

    // let res = seco::models::products::Entity::find().all(&ctx.db).await.unwrap();
    // println!("{:?}", res);

    #[derive(FromQueryResult, Debug)]
    struct ProductAndSellerResponse {
        id: i32,
        title: String,
        seller_pid: Uuid,
        seller_first_name: String,
        seller_last_name: String,
        seller_joined_date: DateTime,
    }

    // let product_and_users: Vec<(ProductModel, Option<UserModel>)> = Products::find().find_also_related(Users).all(&ctx.db).await.unwrap();
    // println!("{:?}", product_and_users);
    let product_sellers: Vec<ProductAndSellerResponse> = Products::find()
        .select_only()
        .column(<seco::models::products::Entity as sea_orm::EntityTrait>::Column::Id)
        .column(<seco::models::products::Entity as sea_orm::EntityTrait>::Column::Title)
        .column_as(<seco::models::users::Entity as sea_orm::EntityTrait>::Column::Pid, "seller_pid")
        .column_as(<seco::models::users::Entity as sea_orm::EntityTrait>::Column::FirstName, "seller_first_name")
        .column_as(<seco::models::users::Entity as sea_orm::EntityTrait>::Column::LastName, "seller_last_name")
        .column_as(<seco::models::users::Entity as sea_orm::EntityTrait>::Column::CreatedAt, "seller_joined_date")
        .join(JoinType::InnerJoin, <seco::models::products::Entity as sea_orm::EntityTrait>::Relation::Users.def())
        .into_model::<ProductAndSellerResponse>()
        .all(&ctx.db)
        .await?;
    println!("{:?}", product_sellers);
    println!("welcome to playground. edit me at `examples/playground.rs`");

    Ok(())
}
