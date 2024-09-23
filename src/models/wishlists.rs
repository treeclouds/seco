use loco_rs::model::ModelResult;
use sea_orm::entity::prelude::*;
use sea_orm::{FromQueryResult, DbBackend, JsonValue, Statement, ActiveValue};
use crate::views::wishlist::WishlistListResponse;
use super::_entities::wishlists::{ActiveModel, Model};

impl ActiveModelBehavior for ActiveModel {
    // extend activemodel below (keep comment for generators)
}

impl super::_entities::wishlists::Model {
    pub async fn get_wishlist_by_user_id(db: &DatabaseConnection, user_id: &i32)  -> ModelResult<Vec<WishlistListResponse>> {
        let products: Vec<WishlistListResponse> = JsonValue::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"
            SELECT
                w.id,
                COALESCE(
                    json_build_object(
                        'id', p.id,
                        'title', p.title,
                        'price', p.price,
                        'images', COALESCE((
                           SELECT json_agg(json_build_object('id', pi2.id, 'image', pi2.image))
                           FROM product_images pi2 where pi2.product_id = p.id
                        ), '[]'::json),
                        'seller', COALESCE (
                            json_build_object(
                                'pid', u.pid,
                                'first_name', u.first_name,
                                'last_name', u.last_name,
                                'joined_date', u.created_at,
                                'location', u.location
                            ), '{}'::json
                        )
                    ), '{}'::json
                ) as product_detail
            FROM wishlists w
            INNER JOIN products p ON p.id = w.product_id
            INNER JOIN users u ON u.id = w.user_id
            WHERE u.id = $1 AND w.is_deleted = false
            GROUP BY w.id, p.id, u.pid, u.first_name, u.last_name, u.created_at, u.location
        "#,
            [(*user_id).into()],
        )).into_model::<WishlistListResponse>()
            .all(db)
            .await?;
        Ok(products)
    }
}

impl super::_entities::wishlists::ActiveModel {
    pub async fn set_wishlist_deleted(
        mut self,
        db: &DatabaseConnection,
    ) -> ModelResult<Model> {
        self.is_deleted = ActiveValue::set(true);
        Ok(self.update(db).await?)
    }
}
