use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "metrics")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(column_name = "report_id")]
    pub report_id: Uuid,
    #[sea_orm(column_name = "benchmark_id")]
    pub benchmark_id: Uuid,
    #[sea_orm(column_name = "measure_id")]
    pub measure_id: Uuid,
    pub value: f64,
    pub lower: Option<f64>,
    pub upper: Option<f64>,
    #[sea_orm(column_name = "created_at")]
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::report::Entity",
        from = "Column::ReportId",
        to = "super::report::Column::Id"
    )]
    Report,
    #[sea_orm(
        belongs_to = "super::benchmark::Entity",
        from = "Column::BenchmarkId",
        to = "super::benchmark::Column::Id"
    )]
    Benchmark,
    #[sea_orm(
        belongs_to = "super::measure::Entity",
        from = "Column::MeasureId",
        to = "super::measure::Column::Id"
    )]
    Measure,
    #[sea_orm(has_many = "super::alert::Entity")]
    Alerts,
}

impl Related<super::report::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Report.def()
    }
}

impl Related<super::benchmark::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Benchmark.def()
    }
}

impl Related<super::measure::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Measure.def()
    }
}

impl Related<super::alert::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Alerts.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
