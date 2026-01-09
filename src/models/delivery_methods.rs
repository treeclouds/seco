use loco_rs::model::ModelResult;
use sea_orm::entity::prelude::*;
use sea_orm::{FromQueryResult, JsonValue, Statement, DbBackend};
use crate::views::delivery_method::DeliveryMethodsAndServicesResponse;
pub use super::_entities::delivery_methods::{ActiveModel, Model, Entity};
pub type DeliveryMethods = Entity;

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
    pub async fn get_all_delivery_methods_and_services(
        db: &DatabaseConnection,
    ) -> ModelResult<Vec<DeliveryMethodsAndServicesResponse>> {
        let query = r#"
            SELECT
                dm.id,
                dm.name,
                dm.active,
                COALESCE(
                    json_agg(
                        json_build_object(
                            'id', dms.id,
                            'name', dms.name,
                            'price', dms.price,
                            'active', dms.active
                        )
                    ) FILTER (WHERE dms.id IS NOT NULL),
                    '[]'
                ) AS services,
                COUNT(dms) as total_services
            FROM delivery_methods dm
            LEFT JOIN delivery_method_services dms ON dms.delivery_method_id = dm.id
            GROUP BY dm.id, dm.name
            ORDER BY dm.name
        "#;

        let products: Vec<DeliveryMethodsAndServicesResponse> = JsonValue::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres, query, [],
        )).into_model::<DeliveryMethodsAndServicesResponse>()
            .all(db)
            .await?;
        Ok(products)
    }
}

// implement your write-oriented logic here
impl ActiveModel {}

// implement your custom finders, selectors oriented logic here
impl Entity {}
