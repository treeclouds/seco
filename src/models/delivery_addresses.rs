use loco_rs::model::{ModelError, ModelResult};
use sea_orm::entity::prelude::*;
use sea_orm::query::*;
pub use super::_entities::delivery_addresses::{self, ActiveModel, Model, Entity};

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
    pub async fn find_by_id_and_user_id(db: &DatabaseConnection, id: i32, user_id: i32) -> ModelResult<Self> {
        tracing::info!("===== find_by_id_and_user_id id {:?}", id);
        tracing::info!("===== find_by_id_and_user_id user_id {:?}", user_id);
        let delivery_address = Entity::find()
            .filter(
                Condition::all()
                    .add(delivery_addresses::Column::Id.eq(id))
                    .add(delivery_addresses::Column::UserId.eq(user_id))
            )
            .one(db).await?;
        delivery_address.ok_or_else(|| ModelError::EntityNotFound)
    }
}

// implement your write-oriented logic here
impl ActiveModel {}

// implement your custom finders, selectors-oriented logic here
impl Entity {}
