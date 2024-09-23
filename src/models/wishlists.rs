use loco_rs::model::ModelResult;
use sea_orm::entity::prelude::*;
use sea_orm::{FromQueryResult, DbBackend, JsonValue, Statement};
use crate::views::product::ProductsResponse;
use super::_entities::wishlists::ActiveModel;

impl ActiveModelBehavior for ActiveModel {
    // extend activemodel below (keep comment for generators)
}

impl super::_entities::wishlists::Model {
    pub async fn get_wishlist_by_user_id(db: &DatabaseConnection, user_id: &i32)  -> ModelResult<Vec<ProductsResponse>> {
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
            FROM wishlists w
            INNER JOIN products p ON p.id = w.product_id
            INNER JOIN users u ON u.id = w.user_id
            WHERE u.id = $1 AND w.is_deleted = false
            GROUP BY p.id, u.pid, u.first_name, u.last_name, u.created_at, u.location
        "#,
            [(*user_id).into()],
        )).into_model::<ProductsResponse>()
            .all(db)
            .await?;
        Ok(products)
    }
}
