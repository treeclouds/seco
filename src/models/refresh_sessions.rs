use chrono::Local;
use loco_rs::model::ModelResult;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue, TransactionTrait};
pub use super::_entities::refresh_sessions::{self, ActiveModel, Model, Entity};
pub type RefreshSessions = Entity;

use sha2::{Sha256, Digest};

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

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
    pub async fn create_refresh_token(
        db: &DatabaseConnection,
        user_id: i32,
        token: &str,
        exp_dt: chrono::DateTime<Local>,
        jti: String,
    ) -> ModelResult<Self> {
        let txn = db.begin().await?;
        let token_hash = hash_token(&token);
        let refresh_session = ActiveModel {
            user_id: ActiveValue::set(user_id),
            jti: ActiveValue::set(jti),
            token_hash: ActiveValue::set(token_hash),
            revoked: ActiveValue::set(false),
            expires_at: ActiveValue::set(exp_dt.into()),
            revoked_at: ActiveValue::set(None),
            ..Default::default()
        }.insert(&txn).await?;

        txn.commit().await?;
        Ok(refresh_session)
    }
}

// implement your write-oriented logic here
impl ActiveModel {}

// implement your custom finders, selectors oriented logic here
impl Entity {}
