use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Copy, Serialize, Deserialize, ToSchema)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "verification_status")]
pub enum VerificationStatus {
    #[sea_orm(string_value = "Pending")]
    Pending,
    #[sea_orm(string_value = "Approved")]
    Approved,
    #[sea_orm(string_value = "Rejected")]
    Rejected,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "seller_verifications")]
pub struct Model {
    pub created_at: DateTime,
    pub updated_at: DateTime,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,
    pub face_photo: String,
    pub ktp_photo: String,
    pub face_ktp_photo: String,
    pub domicile_photo: String,
    pub status: VerificationStatus,
    pub reject_reason: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    User,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
