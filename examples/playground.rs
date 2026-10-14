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
    //             d.id, d.title, d.category_id, d.description, d.price, d.dimension_width,
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
    //         GROUP BY d.id, d.title, d.category_id, d.description, d.price, d.dimension_width,
    //             d.dimension_height, d.dimension_length, d.dimension_weight, d.brand, d.material,
    //             d.stock, d.sku, d.tags::jsonb, d.condition::text, d.created_at, d.pid,
    //             d.first_name, d.last_name, d.user_created
    //     "#,
    //     [],
    // )).into_model::<ProductAndSellerResponse>()
    //     .all(&ctx.db)
    //     .await?;
    // let products: Vec<ProductAndSellerResponse> = JsonValue::find_by_statement(Statement::from_sql_and_values(
    //     DbBackend::Postgres,
    //     r#"
    //         SELECT p.id, p.title, p.category_id, p.description, p.price, p.dimension_width,
    //             p.dimension_height, p.dimension_length, p.dimension_weight, p.brand, p.material,
    //             p.stock, p.sku, p.tags::jsonb, p.condition::text, p.created_at,
    //             COALESCE((
    //                SELECT json_agg(json_build_object('id', pi2.id, 'image', pi2.image))
    //                FROM product_images pi2 where pi2.product_id = p.id
    //             ), '[]'::json) as images,
    //             COALESCE (
    //                 json_agg(json_build_object('pid', u.pid, 'first_name', u.first_name, 'last_name', u.last_name, 'joined_date', u.created_at)), '{}'::json
    //             ) as seller
    //         FROM products p
    //         INNER JOIN users u ON u.id = p.seller_id
    //         GROUP BY p.id;
    //     "#,
    //     [],
    // )).into_model::<ProductAndSellerResponse>()
    //     .all(&ctx.db)
    //     .await?;
    // let product: Option<ProductAndSellerResponse> = products::Entity::find()
    //     .from_raw_sql(Statement::from_sql_and_values(
    //         DbBackend::Postgres,
    //         r#"
    //         SELECT p.id, p.title, p.category_id, p.description, p.price, p.dimension_width,
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
    //         WHERE p.id = $1 AND u.id = $2
    //         GROUP BY p.id, u.pid, u.first_name, u.last_name, u.created_at;
    //         "#,
    //         [5.into(), 3.into()],
    //     ))
    //     .into_model::<ProductAndSellerResponse>()
    //     .one(&ctx.db)
    //     .await?;
    let condition = Some("BrandNew");
    let mut extra_where_clause: String = "WHERE p.seller_id IS NOT NULL".to_owned();
    extra_where_clause += &*format!(" AND p.condition = {:?}", condition.unwrap().to_owned());
    // extra_where_clause = format!("{} {}", extra_where_clause, " AND p.brand = 'a'");
    // extra_where_clause = format!("{} {}", extra_where_clause, " AND p.condition = 'BrandNew'");
    let query = r#"
            SELECT p.id, p.title, p.category_id, p.description, p.price, p.dimension_width,
                p.dimension_height, p.dimension_length, p.dimension_weight, p.brand, p.material,
                p.stock, p.sku, p.tags::jsonb, p.condition::text, p.created_at,
                COALESCE((
                   SELECT json_agg(json_build_object('id', pi2.id, 'image', pi2.image))
                   FROM product_images pi2 where pi2.product_id = p.id
                ), '[]'::json) as images,
                COALESCE (
                    json_build_object(
                        'pid', u.pid,
                        'first_name', u.first_name,
                        'last_name', u.last_name,
                        'joined_date', u.created_at,
                        'location', u.location
                    ), '{}'::json
                ) as seller
            FROM products p
            INNER JOIN users u ON u.id = p.seller_id
            INNER JOIN categories c ON c.id = p.category_id
        "#;
    let group_by: String = "GROUP BY p.id, u.pid, u.first_name, u.last_name, u.created_at, u.location".to_owned();
    let combine_query = format!("{} {} {}", query, extra_where_clause, group_by);
    println!("===== query {}", combine_query);
    let products: Vec<ProductsResponse> = JsonValue::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres, combine_query, [],
    )).into_model::<ProductsResponse>()
        .all(&ctx.db)
        .await?;
    println!("{:?}", products);

    // let query_res_vec: Vec<QueryResult> = ctx.db.query_all(
    //     Statement::from_string(
    //         DbBackend::Postgres,
    //         r#"
    //             SELECT p.id, p.title, p.category_id, p.description, p.price, p.dimension_width,
    //                 p.dimension_height, p.dimension_length, p.dimension_weight, p.brand, p.material,
    //                 p.stock, p.sku, p.tags::jsonb, p.condition::text, p.created_at, p.updated_at
    //                 COALESCE((
    //                    SELECT json_agg(json_build_object('id', pi2.id, 'image', pi2.image))
    //                    FROM product_images pi2 where pi2.product_id = p.id
    //                 ), '[]'::json) as images,
    //                 COALESCE (
    //                     json_build_object('pid', u.pid, 'first_name', u.first_name, 'last_name', u.last_name, 'joined_date', u.created_at), '{}'::json
    //                 ) as seller
    //             FROM products p
    //             INNER JOIN users u ON u.id = p.seller_id
    //             GROUP BY p.id, u.pid, u.first_name, u.last_name, u.created_at
    //         "#,
    //     ))
    //     .await?;
    // let res1 = query::fetch_page(&ctx.db, query_res_vec, &pagination_query).await;
    // println!("{:?}", res);
    println!("welcome to playground. edit me at `examples/playground.rs`");

    Ok(())
}
