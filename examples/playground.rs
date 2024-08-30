use eyre::Context;
#[allow(unused_imports)]
use loco_rs::{cli::playground, prelude::*};
use sea_orm::{FromQueryResult, JoinType, RelationTrait, QuerySelect, Statement, DatabaseBackend, QueryResult, JsonValue, DbBackend};
use sea_orm::prelude::Decimal;
use uuid::Uuid;
use seco::app::App;
use seco::models::_entities::sea_orm_active_enums::Condition;
use seco::models::products;
use seco::views::product::ProductsResponse;

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

    // #[derive(FromQueryResult, Debug)]
    // struct ProductAndSellerResponse {
    //     id: i32,
    //     title: String,
    //     seller_pid: Uuid,
    //     seller_first_name: String,
    //     seller_last_name: String,
    //     seller_joined_date: DateTime,
    // }

    #[derive(FromQueryResult, Debug)]
    struct ProductAndSellerResponse {
        id: i32,
        category: String,
        title: String,
        description: String,
        price: Decimal,
        dimension_width: f32,
        dimension_height: f32,
        dimension_length: f32,
        dimension_weight: f32,
        brand: String,
        material: String,
        stock: i32,
        sku: String,
        tags: Option<JsonValue>,
        condition: Option<String>,
        created_at: DateTime,
        images: Option<JsonValue>,
        seller_pid: Uuid,
        seller_first_name: String,
        seller_last_name: String,
        seller_joined_date: DateTime,
    }

    // let product_and_users: Vec<(ProductModel, Option<UserModel>)> = Products::find().find_also_related(Users).all(&ctx.db).await.unwrap();
    // println!("{:?}", product_and_users);
    // let product_sellers: Vec<ProductAndSellerResponse> = Products::find()
    //     .select_only()
    //     .column(<seco::models::products::Entity as sea_orm::EntityTrait>::Column::Id)
    //     .column(<seco::models::products::Entity as sea_orm::EntityTrait>::Column::Title)
    //     .column_as(<seco::models::users::Entity as sea_orm::EntityTrait>::Column::Pid, "seller_pid")
    //     .column_as(<seco::models::users::Entity as sea_orm::EntityTrait>::Column::FirstName, "seller_first_name")
    //     .column_as(<seco::models::users::Entity as sea_orm::EntityTrait>::Column::LastName, "seller_last_name")
    //     .column_as(<seco::models::users::Entity as sea_orm::EntityTrait>::Column::CreatedAt, "seller_joined_date")
    //     .join(JoinType::InnerJoin, <seco::models::products::Entity as sea_orm::EntityTrait>::Relation::Users.def())
    //     .into_model::<ProductAndSellerResponse>()
    //     .all(&ctx.db)
    //     .await?;
    // println!("{:?}", product_sellers);
    // let query_res_vec: Vec<QueryResult> = ctx.db.query_all(Statement::from_string(DatabaseBackend::Postgres, "SELECT * FROM products;",)).await?;
    // let products: Vec<ProductAndSellerResponse> = JsonValue::find_by_statement(Statement::from_sql_and_values(
    //     DbBackend::Postgres,
    //     r#"
    //         SELECT
    //             d.id, d.title, d.category, d.description, d.price, d.dimension_width,
    //             d.dimension_height, d.dimension_length, d.dimension_weight, d.brand, d.material,
    //             d.stock, d.sku, d.tags::jsonb, d.condition::text, d.created_at,
    //             array_to_json(array_agg(d.image)) AS images,
    //             d.pid AS seller_pid, d.first_name AS seller_first_name,
    //             d.last_name AS seller_last_name, d.user_created AS seller_joined_date
    //         FROM (
    //             SELECT p.*, u.pid, u.first_name, u.last_name, u.created_at as user_created, pi.image
    //             FROM products p
    //             LEFT JOIN users u ON u.id = p.seller_id
    //             INNER JOIN product_images pi ON pi.product_id = p.id
    //         ) d
    //         GROUP BY d.id, d.title, d.category, d.description, d.price, d.dimension_width,
    //             d.dimension_height, d.dimension_length, d.dimension_weight, d.brand, d.material,
    //             d.stock, d.sku, d.tags::jsonb, d.condition::text, d.created_at, d.pid,
    //             d.first_name, d.last_name, d.user_created
    //     "#,
    //     [],
    // )).into_model::<ProductAndSellerResponse>()
    //     .all(&ctx.db)
    //     .await?;
    // let products: Vec<ProductsResponse> = JsonValue::find_by_statement(Statement::from_sql_and_values(
    //     DbBackend::Postgres,
    //     r#"
    //         SELECT p.id, p.title, p.category, p.description, p.price, p.dimension_width,
    //             p.dimension_height, p.dimension_length, p.dimension_weight, p.brand, p.material,
    //             p.stock, p.sku, p.tags::jsonb, p.condition::text, p.created_at,
    //             COALESCE((
    //                SELECT json_agg(json_build_object('id', pi2.id, 'image', pi2.image))
    //                FROM product_images pi2 where pi2.product_id = p.id
    //             ), '[]'::json) as images,
    //             u.pid AS seller_pid, u.first_name AS seller_first_name,
    //             u.last_name AS seller_last_name, u.created_at AS seller_joined_date
    //         FROM products p
    //         INNER JOIN users u ON u.id = p.seller_id
    //         GROUP BY p.id, u.pid, u.first_name, u.last_name, u.created_at;
    //     "#,
    //     [],
    // )).into_model::<ProductsResponse>()
    //     .all(&ctx.db)
    //     .await?;
    let product: Option<ProductsResponse> = products::Entity::find()
        .from_raw_sql(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"
            SELECT p.id, p.title, p.category, p.description, p.price, p.dimension_width,
                p.dimension_height, p.dimension_length, p.dimension_weight, p.brand, p.material,
                p.stock, p.sku, p.tags::jsonb, p.condition::text, p.created_at,
                COALESCE((
                   SELECT json_agg(json_build_object('id', pi2.id, 'image', pi2.image))
                   FROM product_images pi2 where pi2.product_id = p.id
                ), '[]'::json) as images,
                u.pid AS seller_pid, u.first_name AS seller_first_name,
                u.last_name AS seller_last_name, u.created_at AS seller_joined_date
            FROM products p
            INNER JOIN users u ON u.id = p.seller_id
            WHERE p.id = $1 AND u.id = $2
            GROUP BY p.id, u.pid, u.first_name, u.last_name, u.created_at;
            "#,
            [5.into(), 3.into()],
        ))
        .into_model::<ProductsResponse>()
        .one(&ctx.db)
        .await?;
    println!("{:?}", product);
    println!("welcome to playground. edit me at `examples/playground.rs`");

    Ok(())
}
