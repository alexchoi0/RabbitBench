use async_graphql::dataloader::DataLoader;
use async_graphql::{ComplexObject, Context, Enum, Result, SimpleObject, ID};
use uuid::Uuid;

use crate::entities::alert::AlertStatus as DbAlertStatus;
use crate::loaders::{MetricLoader, ThresholdLoader};

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum AlertStatusInput {
    Active,
    Acknowledged,
    Resolved,
}

impl AlertStatusInput {
    pub fn to_db_value(&self) -> DbAlertStatus {
        match self {
            AlertStatusInput::Active => DbAlertStatus::Active,
            AlertStatusInput::Acknowledged => DbAlertStatus::Acknowledged,
            AlertStatusInput::Resolved => DbAlertStatus::Resolved,
        }
    }
}

#[derive(SimpleObject)]
#[graphql(complex, cache_control(max_age = 60))]
pub struct Alert {
    pub id: ID,
    pub status: String,
    pub percent_change: f64,
    pub baseline_value: f64,
    pub current_value: f64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[graphql(skip)]
    pub metric_id: Uuid,
    #[graphql(skip)]
    pub threshold_id: Uuid,
}

impl From<crate::entities::alert::Model> for Alert {
    fn from(model: crate::entities::alert::Model) -> Self {
        let status = match model.status {
            DbAlertStatus::Active => "active",
            DbAlertStatus::Acknowledged => "acknowledged",
            DbAlertStatus::Resolved => "resolved",
        };

        Self {
            id: ID(model.id.to_string()),
            status: status.to_string(),
            percent_change: model.percent_change,
            baseline_value: model.baseline_value,
            current_value: model.current_value,
            created_at: model.created_at.into(),
            metric_id: model.metric_id,
            threshold_id: model.threshold_id,
        }
    }
}

#[ComplexObject]
impl Alert {
    async fn metric(&self, ctx: &Context<'_>) -> Result<super::Metric> {
        let loader = ctx.data::<DataLoader<MetricLoader>>()?;
        loader
            .load_one(self.metric_id)
            .await?
            .ok_or_else(|| "Metric not found".into())
    }

    async fn threshold(&self, ctx: &Context<'_>) -> Result<super::Threshold> {
        let loader = ctx.data::<DataLoader<ThresholdLoader>>()?;
        loader
            .load_one(self.threshold_id)
            .await?
            .ok_or_else(|| "Threshold not found".into())
    }
}
