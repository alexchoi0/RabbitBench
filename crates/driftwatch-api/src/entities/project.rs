use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "projects")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    #[sea_orm(unique)]
    pub slug: String,
    pub name: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    pub public: bool,
    #[sea_orm(nullable)]
    pub github_repo: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub github_token: Option<String>,
    pub github_pr_comments: bool,
    pub github_status_checks: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::branch::Entity")]
    Branches,
    #[sea_orm(has_many = "super::testbed::Entity")]
    Testbeds,
    #[sea_orm(has_many = "super::measure::Entity")]
    Measures,
    #[sea_orm(has_many = "super::benchmark::Entity")]
    Benchmarks,
    #[sea_orm(has_many = "super::report::Entity")]
    Reports,
    #[sea_orm(has_many = "super::threshold::Entity")]
    Thresholds,
}

impl Related<super::branch::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Branches.def()
    }
}

impl Related<super::testbed::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Testbeds.def()
    }
}

impl Related<super::measure::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Measures.def()
    }
}

impl Related<super::benchmark::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Benchmarks.def()
    }
}

impl Related<super::report::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Reports.def()
    }
}

impl Related<super::threshold::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Thresholds.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
