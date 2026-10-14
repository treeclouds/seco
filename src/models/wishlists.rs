use loco_rs::model::ModelResult;
use sea_orm::entity::prelude::*;
use sea_orm::{FromQueryResult, DbBackend, JsonValue, Statement, ActiveValue};
use crate::views::wishlist::WishlistListResponse;
use super::_entities::wishlists::{ActiveModel, Model};

impl ActiveModelBehavior for ActiveModel {
    // extend activemodel below (keep comment for generators)
}

impl Model {
    pub async fn get_wishlist_by_user_id(db: &DatabaseConnection, user_id: &i32)  -> ModelResult<Vec<WishlistListResponse>> {
        let products: Vec<WishlistListResponse> = JsonValue::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"
            SELECT
                w.id, p.id as product_id, p.title, p.category_id, p.description, p.price,
                p.dimension_width, p.dimension_height, p.dimension_length, p.dimension_weight,
                p.brand_id, p.material_id, p.stock, p.sku, p.tags::jsonb, p.condition::text, p.created_at,
                COALESCE((
                   SELECT json_agg(json_build_object('id', pi2.id, 'image', 'media/' || pi2.image))
                   FROM product_images pi2 where pi2.product_id = p.id
                ), '[]'::json) as images,
                COALESCE (
                    json_build_object(
                        'pid', s.pid,
                        'first_name', s.first_name,
                        'last_name', s.last_name,
                        'joined_date', s.created_at,
                        'location', s.location
                    ), '{}'::json
                ) as seller
            FROM wishlists w
            INNER JOIN products p ON p.id = w.product_id
            INNER JOIN users u ON u.id = w.user_id
            INNER JOIN users s ON s.id = p.seller_id
            LEFT JOIN brands b ON b.id = p.brand_id
            LEFT JOIN materials m ON m.id = p.material_id
            WHERE w.user_id = $1 AND w.is_deleted = false
            GROUP BY w.id, p.id, s.pid, s.first_name, s.last_name, s.created_at, s.location
        "#,
            [(*user_id).into()],
        )).into_model::<WishlistListResponse>()
            .all(db)
            .await?;
        Ok(products)
    }
}

impl ActiveModel {
    pub async fn set_wishlist_deleted(
        mut self,
        db: &DatabaseConnection,
    ) -> ModelResult<Model> {
        self.is_deleted = ActiveValue::set(true);
        Ok(self.update(db).await?)
    }

    pub async fn set_wishlist_un_deleted(
        mut self,
        db: &DatabaseConnection,
    ) -> ModelResult<Model> {
        self.is_deleted = ActiveValue::set(false);
        Ok(self.update(db).await?)
    }
}
