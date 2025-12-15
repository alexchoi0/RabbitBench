use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "reports")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(column_name = "project_id")]
    pub project_id: Uuid,
    #[sea_orm(column_name = "branch_id")]
    pub branch_id: Uuid,
    #[sea_orm(column_name = "testbed_id")]
    pub testbed_id: Uuid,
    #[sea_orm(column_name = "git_hash", nullable)]
    pub git_hash: Option<String>,
    #[sea_orm(column_name = "pr_number", nullable)]
    pub pr_number: Option<i32>,
    #[sea_orm(column_name = "created_at")]
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::project::Entity",
        from = "Column::ProjectId",
        to = "super::project::Column::Id"
    )]
    Project,
    #[sea_orm(
        belongs_to = "super::branch::Entity",
        from = "Column::BranchId",
        to = "super::branch::Column::Id"
    )]
    Branch,
    #[sea_orm(
        belongs_to = "super::testbed::Entity",
        from = "Column::TestbedId",
        to = "super::testbed::Column::Id"
    )]
    Testbed,
    #[sea_orm(has_many = "super::metric::Entity")]
    Metrics,
    #[sea_orm(has_many = "super::flamegraph::Entity")]
    Flamegraphs,
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl Related<super::branch::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Branch.def()
    }
}

impl Related<super::testbed::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Testbed.def()
    }
}

impl Related<super::metric::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Metrics.def()
    }
}

impl Related<super::flamegraph::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Flamegraphs.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
