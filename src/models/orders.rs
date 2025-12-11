use loco_rs::model::ModelResult;
use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;
use crate::models::_entities::sea_orm_active_enums::OrderStatusEnum;
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
impl Model {}

// implement your write-oriented logic here
impl ActiveModel {
    pub async fn set_status_completed(
        mut self,
        db: &DatabaseConnection,
    ) -> ModelResult<Model> {
        self.status = ActiveValue::set(OrderStatusEnum::Completed);
        Ok(self.update(db).await?)
    }
}

// implement your custom finders, selectors oriented logic here
impl Entity {}
