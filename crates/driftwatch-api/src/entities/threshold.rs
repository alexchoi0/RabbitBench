use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "thresholds")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(column_name = "project_id")]
    pub project_id: Uuid,
    #[sea_orm(column_name = "measure_id")]
    pub measure_id: Uuid,
    #[sea_orm(column_name = "branch_id", nullable)]
    pub branch_id: Option<Uuid>,
    #[sea_orm(column_name = "testbed_id", nullable)]
    pub testbed_id: Option<Uuid>,
    #[sea_orm(column_name = "upper_boundary", nullable)]
    pub upper_boundary: Option<f64>,
    #[sea_orm(column_name = "lower_boundary", nullable)]
    pub lower_boundary: Option<f64>,
    #[sea_orm(column_name = "min_sample_size")]
    pub min_sample_size: i32,
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
    #[sea_orm(
        belongs_to = "super::measure::Entity",
        from = "Column::MeasureId",
        to = "super::measure::Column::Id"
    )]
    Measure,
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
    #[sea_orm(has_many = "super::alert::Entity")]
    Alerts,
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl Related<super::measure::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Measure.def()
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

impl Related<super::alert::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Alerts.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
