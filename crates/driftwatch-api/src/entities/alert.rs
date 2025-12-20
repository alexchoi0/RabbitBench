use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "alert_status")]
pub enum AlertStatus {
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "acknowledged")]
    Acknowledged,
    #[sea_orm(string_value = "resolved")]
    Resolved,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "alerts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(column_name = "threshold_id")]
    pub threshold_id: Uuid,
    #[sea_orm(column_name = "metric_id")]
    pub metric_id: Uuid,
    pub status: AlertStatus,
    #[sea_orm(column_name = "percent_change")]
    pub percent_change: f64,
    #[sea_orm(column_name = "baseline_value")]
    pub baseline_value: f64,
    #[sea_orm(column_name = "current_value")]
    pub current_value: f64,
    #[sea_orm(column_name = "created_at")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(column_name = "updated_at")]
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::threshold::Entity",
        from = "Column::ThresholdId",
        to = "super::threshold::Column::Id"
    )]
    Threshold,
    #[sea_orm(
        belongs_to = "super::metric::Entity",
        from = "Column::MetricId",
        to = "super::metric::Column::Id"
    )]
    Metric,
}

impl Related<super::threshold::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Threshold.def()
    }
}

impl Related<super::metric::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Metric.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
