use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "flamegraphs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(column_name = "report_id")]
    pub report_id: Uuid,
    #[sea_orm(column_name = "benchmark_id", nullable)]
    pub benchmark_id: Option<Uuid>,
    pub filename: String,
    #[sea_orm(column_name = "storage_path")]
    pub storage_path: String,
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

impl ActiveModelBehavior for ActiveModel {}
