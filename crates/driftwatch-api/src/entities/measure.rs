use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "measures")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(column_name = "project_id")]
    pub project_id: Uuid,
    pub name: String,
    pub units: Option<String>,
    #[sea_orm(column_name = "created_at")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(column_name = "updated_at")]
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::project::Entity",
        from = "Column::ProjectId",
        to = "super::project::Column::Id"
    )]
    Project,
    #[sea_orm(has_many = "super::metric::Entity")]
    Metrics,
    #[sea_orm(has_many = "super::threshold::Entity")]
    Thresholds,
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl Related<super::metric::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Metrics.def()
    }
}

impl Related<super::threshold::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Thresholds.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
