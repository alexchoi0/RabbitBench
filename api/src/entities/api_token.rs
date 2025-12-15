use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "api_tokens")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(column_name = "user_id")]
    pub user_id: String,
    pub name: String,
    #[sea_orm(column_name = "token_hash", column_type = "Text")]
    pub token_hash: String,
    #[sea_orm(column_name = "last_used_at", nullable)]
    pub last_used_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(column_name = "created_at")]
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
