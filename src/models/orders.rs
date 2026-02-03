use loco_rs::model::ModelResult;
use sea_orm::{FromQueryResult, JsonValue, Statement, DbBackend, ActiveValue};
use sea_orm::entity::prelude::*;
use crate::models::_entities::sea_orm_active_enums::OrderStatusEnum;
use crate::views::order::OrderDetailResponse;
pub use super::_entities::orders::{ActiveModel, Model, Entity};
pub type Orders = Entity;

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
    pub async fn get_order_by_buyer_id_and_order_number(
        db: &DatabaseConnection,
        buyer_id: &i32,
        order_numer: &String
    ) -> Result<OrderDetailResponse, loco_rs::Error> {
        let query = r#"
            SELECT o.order_number,
               o.final_price,
               CASE
                   WHEN o.delivery_address_detail IS NULL OR btrim(o.delivery_address_detail) = '' THEN '{}'::jsonb
                   ELSE o.delivery_address_detail::jsonb
                   END AS delivery_address_detail,
               COALESCE(o.payment_method_detail, '{}'::json)::jsonb as payment_method_detail,
               COALESCE(o.meetup_address_detail, '{}'::json)::jsonb as meetup_address_detail,
               o.status::text,
               COALESCE(
                   jsonb_agg(
                       jsonb_build_object(
                           'product_id', oi.product_id,
                           'product_name', oi.product_name,
                           'product_price', oi.product_price,
                           'product_sku', oi.product_sku,
                           'qty', oi.qty,
                           'images', COALESCE(
                               (
                                    SELECT json_agg(json_build_object('id', pi2.id, 'image', 'media/' || pi2.image))
                                    FROM product_images pi2 where pi2.product_id = p.id
                               ), '[]'::json
                           ),
                           'seller', COALESCE(
                               jsonb_build_object(
                                   'pid', s.pid,
                                   'first_name', s.first_name,
                                   'last_name', s.last_name,
                                   'joined_date', s.created_at,
                                   'location', s.location
                               ), '{}'::jsonb
                            )
                       )
                    ) FILTER (WHERE oi.order_id IS NOT NULL),
                   '[]'::jsonb
               ) as order_items
            FROM orders o
            INNER JOIN order_items oi ON o.id = oi.order_id
            LEFT JOIN products p ON p.id = oi.product_id
            LEFT JOIN users s ON s.id = oi.seller_id
            WHERE o.buyer_id = $1 AND o.order_number = $2
            GROUP BY o.order_number, o.final_price, o.delivery_address_detail, COALESCE(o.payment_method_detail, '{}'::json)::jsonb, COALESCE(o.meetup_address_detail, '{}'::json)::jsonb, o.status
        "#.to_string();
        let values = vec![(*buyer_id).into(), order_numer.clone().into()];
        let order: Option<OrderDetailResponse> = JsonValue::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres,
            query,
            values,
        ))
            .into_model::<OrderDetailResponse>()
            .one(db)
            .await?;

        order.ok_or_else(|| loco_rs::Error::NotFound)
    }

    pub async fn get_all_orders_by_buyer_id(
        db: &DatabaseConnection,
        buyer_id: &i32
    ) -> ModelResult<Vec<OrderDetailResponse>> {
        let query = r#"
            SELECT o.order_number,
               o.final_price,
               CASE
                   WHEN o.delivery_address_detail IS NULL OR btrim(o.delivery_address_detail) = '' THEN '{}'::jsonb
                   ELSE o.delivery_address_detail::jsonb
                   END AS delivery_address_detail,
               COALESCE(o.payment_method_detail, '{}'::json)::jsonb as payment_method_detail,
               COALESCE(o.meetup_address_detail, '{}'::json)::jsonb as meetup_address_detail,
               o.status::text,
               COALESCE(
                   jsonb_agg(
                       jsonb_build_object(
                           'product_id', oi.product_id,
                           'product_name', oi.product_name,
                           'product_price', oi.product_price,
                           'product_sku', oi.product_sku,
                           'qty', oi.qty,
                           'images', COALESCE(
                               (
                                    SELECT json_agg(json_build_object('id', pi2.id, 'image', 'media/' || pi2.image))
                                    FROM product_images pi2 where pi2.product_id = p.id
                               ), '[]'::json
                           ),
                           'seller', COALESCE(
                               jsonb_build_object(
                                   'pid', s.pid,
                                   'first_name', s.first_name,
                                   'last_name', s.last_name,
                                   'joined_date', s.created_at,
                                   'location', s.location
                               ), '{}'::jsonb
                            )
                       )
                    ) FILTER (WHERE oi.order_id IS NOT NULL),
                   '[]'::jsonb
               ) as order_items
            FROM orders o
            INNER JOIN order_items oi ON o.id = oi.order_id
            LEFT JOIN products p ON p.id = oi.product_id
            LEFT JOIN users s ON s.id = oi.seller_id
            WHERE o.buyer_id = $1
            GROUP BY o.order_number, o.final_price, o.delivery_address_detail, COALESCE(o.payment_method_detail, '{}'::json)::jsonb, COALESCE(o.meetup_address_detail, '{}'::json)::jsonb, o.status
        "#.to_string();
        let values = vec![(*buyer_id).into()];
        let orders: Vec<OrderDetailResponse> = JsonValue::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres,
            query,
            values,
        ))
            .into_model::<OrderDetailResponse>()
            .all(db)
            .await?;
        Ok(orders)
    }

    pub async fn get_all_orders_by_seller_id(
        db: &DatabaseConnection,
        seller_id: &i32
    ) -> ModelResult<Vec<OrderDetailResponse>> {
        let query = r#"
            SELECT o.order_number,
               o.final_price,
               CASE
                   WHEN o.delivery_address_detail IS NULL OR btrim(o.delivery_address_detail) = '' THEN '{}'::jsonb
                   ELSE o.delivery_address_detail::jsonb
                   END AS delivery_address_detail,
               COALESCE(o.payment_method_detail, '{}'::json)::jsonb as payment_method_detail,
               COALESCE(o.meetup_address_detail, '{}'::json)::jsonb as meetup_address_detail,
               o.status::text,
               COALESCE(
                   jsonb_agg(
                       jsonb_build_object(
                           'product_id', oi.product_id,
                           'product_name', oi.product_name,
                           'product_price', oi.product_price,
                           'product_sku', oi.product_sku,
                           'qty', oi.qty,
                           'images', COALESCE(
                               (
                                    SELECT json_agg(json_build_object('id', pi2.id, 'image', 'media/' || pi2.image))
                                    FROM product_images pi2 where pi2.product_id = p.id
                               ), '[]'::json
                           ),
                           'seller', COALESCE(
                               jsonb_build_object(
                                   'pid', s.pid,
                                   'first_name', s.first_name,
                                   'last_name', s.last_name,
                                   'joined_date', s.created_at,
                                   'location', s.location
                               ), '{}'::jsonb
                            )
                       )
                    ) FILTER (WHERE oi.order_id IS NOT NULL),
                   '[]'::jsonb
               ) as order_items
            FROM orders o
            INNER JOIN order_items oi ON o.id = oi.order_id
            LEFT JOIN products p ON p.id = oi.product_id
            LEFT JOIN users s ON s.id = oi.seller_id
            WHERE o.seller_id = $1
            GROUP BY o.order_number, o.final_price, o.delivery_address_detail, COALESCE(o.payment_method_detail, '{}'::json)::jsonb, COALESCE(o.meetup_address_detail, '{}'::json)::jsonb, o.status
        "#.to_string();
        let values = vec![(*seller_id).into()];
        let orders: Vec<OrderDetailResponse> = JsonValue::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres,
            query,
            values,
        ))
            .into_model::<OrderDetailResponse>()
            .all(db)
            .await?;
        Ok(orders)
    }
    pub async fn get_order_by_seller_id_and_order_number(
        db: &DatabaseConnection,
        seller_id: &i32,
        order_numer: &String
    ) -> Result<OrderDetailResponse, loco_rs::Error> {
        let query = r#"
            SELECT o.order_number,
               o.final_price,
               CASE
                   WHEN o.delivery_address_detail IS NULL OR btrim(o.delivery_address_detail) = '' THEN '{}'::jsonb
                   ELSE o.delivery_address_detail::jsonb
                   END AS delivery_address_detail,
               COALESCE(o.payment_method_detail, '{}'::json)::jsonb as payment_method_detail,
               COALESCE(o.meetup_address_detail, '{}'::json)::jsonb as meetup_address_detail,
               o.status::text,
               COALESCE(
                   jsonb_agg(
                       jsonb_build_object(
                           'product_id', oi.product_id,
                           'product_name', oi.product_name,
                           'product_price', oi.product_price,
                           'product_sku', oi.product_sku,
                           'qty', oi.qty,
                           'images', COALESCE(
                               (
                                    SELECT json_agg(json_build_object('id', pi2.id, 'image', 'media/' || pi2.image))
                                    FROM product_images pi2 where pi2.product_id = p.id
                               ), '[]'::json
                           ),
                           'seller', COALESCE(
                               jsonb_build_object(
                                   'pid', s.pid,
                                   'first_name', s.first_name,
                                   'last_name', s.last_name,
                                   'joined_date', s.created_at,
                                   'location', s.location
                               ), '{}'::jsonb
                            )
                       )
                    ) FILTER (WHERE oi.order_id IS NOT NULL),
                   '[]'::jsonb
               ) as order_items
            FROM orders o
            INNER JOIN order_items oi ON o.id = oi.order_id
            LEFT JOIN products p ON p.id = oi.product_id
            LEFT JOIN users s ON s.id = oi.seller_id
            WHERE o.seller_id = $1 AND o.order_number = $2
            GROUP BY o.order_number, o.final_price, o.delivery_address_detail, COALESCE(o.payment_method_detail, '{}'::json)::jsonb, COALESCE(o.meetup_address_detail, '{}'::json)::jsonb, o.status
        "#.to_string();
        let values = vec![(*seller_id).into(), order_numer.clone().into()];
        let order: Option<OrderDetailResponse> = JsonValue::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres,
            query,
            values,
        ))
            .into_model::<OrderDetailResponse>()
            .one(db)
            .await?;

        order.ok_or_else(|| loco_rs::Error::NotFound)
    }
}

// implement your write-oriented logic here
impl ActiveModel {

    pub async fn set_status_completed(
        mut self,
        db: &DatabaseConnection,
    ) -> ModelResult<Model> {
        self.status = ActiveValue::set(OrderStatusEnum::Completed);
        Ok(self.update(db).await?)
    }

    pub async fn set_status_cancelled(
        mut self,
        db: &DatabaseConnection,
    ) -> ModelResult<Model> {
        self.status = ActiveValue::set(OrderStatusEnum::Cancelled);
        Ok(self.update(db).await?)
    }

    pub async fn set_status_returned(
        mut self,
        db: &DatabaseConnection,
    ) -> ModelResult<Model> {
        self.status = ActiveValue::set(OrderStatusEnum::Returned);
        Ok(self.update(db).await?)
    }
}

// implement your custom finders, selectors oriented logic here
impl Entity {}
