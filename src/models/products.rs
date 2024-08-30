use loco_rs::model::ModelResult;
use sea_orm::entity::prelude::*;
use sea_orm::{FromQueryResult, JsonValue, Statement, DbBackend};
use crate::views::product::ProductsResponse;
pub use super::_entities::products::{self, ActiveModel, Entity, Model};

#[async_trait::async_trait]
impl ActiveModelBehavior for super::_entities::products::ActiveModel {
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
            WHERE p.id = $1
            GROUP BY p.id, u.pid, u.first_name, u.last_name, u.created_at;
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
    ) -> ModelResult<Vec<ProductsResponse>> {
        let products: Vec<ProductsResponse> = JsonValue::find_by_statement(Statement::from_sql_and_values(
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
            GROUP BY p.id, u.pid, u.first_name, u.last_name, u.created_at;
        "#,
            [],
        )).into_model::<ProductsResponse>()
            .all(db)
            .await?;
        Ok(products)
    }

    pub async fn get_all_products_by_user_id(
        db: &DatabaseConnection,
        user_id: &i32
    ) -> ModelResult<Vec<ProductsResponse>> {
        let products: Vec<ProductsResponse> = JsonValue::find_by_statement(Statement::from_sql_and_values(
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
            WHERE u.id = $1
            GROUP BY p.id, u.pid, u.first_name, u.last_name, u.created_at;
        "#,
            [(*user_id).into()],
        )).into_model::<ProductsResponse>()
            .all(db)
            .await?;
        Ok(products)
    }
}

impl super::_entities::products::ActiveModel {}
