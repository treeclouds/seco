use loco_rs::model::ModelResult;
use sea_orm::entity::prelude::*;
use sea_orm::{FromQueryResult, DbBackend, JsonValue, Statement, ActiveValue};
use crate::views::cart::CartListResponse;
pub use super::_entities::carts::{ActiveModel, Model, Entity};
pub type Carts = Entity;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert && self.updated_at.is_unchanged() {
            let mut this = self;
            this.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now().into());
            Ok(this)
        } else {
            Ok(self)
        }
    }
}

// implement your read-oriented logic here
impl Model {
    pub async fn get_cart_by_user_id(db: &DatabaseConnection, user_id: &i32)  -> ModelResult<Vec<CartListResponse>> {
        let products: Vec<CartListResponse> = JsonValue::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"
            SELECT
                c.id, p.id as product_id, p.title, p.category_id, p.description, p.price,
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
                ) as seller,
                c.qty
            FROM cart c
            INNER JOIN products p ON p.id = c.product_id
            INNER JOIN users u ON u.id = c.user_id
            INNER JOIN users s ON s.id = p.seller_id
            LEFT JOIN brands b ON b.id = p.brand_id
            LEFT JOIN materials m ON m.id = p.material_id
            WHERE c.user_id = $1
            GROUP BY c.id, p.id, s.pid, s.first_name, s.last_name, s.created_at, s.location
        "#,
            [(*user_id).into()],
        )).into_model::<CartListResponse>()
            .all(db)
            .await?;
        Ok(products)
    }
}

// implement your write-oriented logic here
impl ActiveModel {}

// implement your custom finders, selectors oriented logic here
impl Entity {}
