use loco_rs::model::ModelResult;
use sea_orm::entity::prelude::*;
use sea_orm::{FromQueryResult, JsonValue, Statement, DbBackend};
use crate::models::_entities::sea_orm_active_enums::{Condition as ProductConditionEnum};
use crate::views::product::ProductsResponse;
pub use super::_entities::products::{self, ActiveModel, Entity, Model};

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    // extend activemodel below (keep comment for generators)
}

impl super::_entities::products::Model {

    pub async fn get_product_by_id(
        db: &DatabaseConnection,
        product_id: &i32
    ) -> ModelResult<Option<ProductsResponse>> {
        let product: Option<ProductsResponse> = products::Entity::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
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
            WHERE p.id = $1
            GROUP BY p.id, u.pid, u.first_name, u.last_name, u.created_at, u.location
            "#,
                [(*product_id).into()],
            ))
            .into_model::<ProductsResponse>()
            .one(db)
            .await?;
        Ok(product)
    }

    pub async fn get_all_products(
        db: &DatabaseConnection,
        condition: &Option<&ProductConditionEnum>,
        location: &Option<&String>,
        brand: &Option<&String>,
        category: &Option<&String>,
    ) -> ModelResult<Vec<ProductsResponse>> {
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
        let mut extra_where_clause: String = "WHERE p.seller_id IS NOT NULL".to_owned();
        if condition.is_some() {
            extra_where_clause += &*format!(" AND p.condition = '{:?}'", condition.unwrap().to_owned());
        }

        if location.is_some() {
            extra_where_clause += &*format!(" AND LOWER(u.location) = LOWER('{:?}')", location.unwrap().to_owned()).replace("\"", "")
        }

        if brand.is_some() {
            extra_where_clause += &*format!(" AND LOWER(p.brand) = LOWER('{:?}')", brand.unwrap().to_owned()).replace("\"", "")
        }

        if category.is_some() {
            extra_where_clause += &*format!(" AND LOWER(c.name) = LOWER('{:?}')", category.unwrap().to_owned()).replace("\"", "")
        }

        let group_by: String = "GROUP BY p.id, u.pid, u.first_name, u.last_name, u.created_at, u.location".to_owned();
        let combine_query = format!("{} {} {}", query, extra_where_clause, group_by);

        println!("===== query {}", combine_query);

        let products: Vec<ProductsResponse> = JsonValue::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres, combine_query, [],
        )).into_model::<ProductsResponse>()
            .all(db)
            .await?;
        Ok(products)
    }

    pub async fn get_product_by_id_and_user_id(
        db: &DatabaseConnection,
        product_id: &i32,
        user_id: &i32
    ) -> Result<ProductsResponse, loco_rs::Error> {
        let product: Option<ProductsResponse> = products::Entity::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
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
            WHERE p.id = $1 AND u.id = $2
            GROUP BY p.id, u.pid, u.first_name, u.last_name, u.created_at, u.location
            "#,
                [(*product_id).into(), (*user_id).into()],
            ))
            .into_model::<ProductsResponse>()
            .one(db)
            .await?;
        product.ok_or_else(|| loco_rs::Error::NotFound)
    }

    pub async fn get_all_products_by_user_id(
        db: &DatabaseConnection,
        user_id: &i32
    ) -> ModelResult<Vec<ProductsResponse>> {
        let products: Vec<ProductsResponse> = JsonValue::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"
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
            WHERE u.id = $1
            GROUP BY p.id, u.pid, u.first_name, u.last_name, u.created_at, u.location
        "#,
            [(*user_id).into()],
        )).into_model::<ProductsResponse>()
            .all(db)
            .await?;
        Ok(products)
    }
}

impl super::_entities::products::ActiveModel {}
