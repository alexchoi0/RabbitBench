use async_graphql::dataloader::DataLoader;
use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use uuid::Uuid;

use crate::loaders::{BenchmarkLoader, MeasureLoader};

#[derive(SimpleObject, Clone)]
#[graphql(complex, cache_control(max_age = 3600))]
pub struct Metric {
    pub id: ID,
    pub value: f64,
    pub lower: Option<f64>,
    pub upper: Option<f64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[graphql(skip)]
    pub benchmark_id: Uuid,
    #[graphql(skip)]
    pub measure_id: Uuid,
}

impl From<crate::entities::metric::Model> for Metric {
    fn from(model: crate::entities::metric::Model) -> Self {
        Self {
            id: ID(model.id.to_string()),
            value: model.value,
            lower: model.lower,
            upper: model.upper,
            created_at: model.created_at.into(),
            benchmark_id: model.benchmark_id,
            measure_id: model.measure_id,
        }
    }
}

#[ComplexObject]
impl Metric {
    async fn benchmark(&self, ctx: &Context<'_>) -> Result<super::Benchmark> {
        let loader = ctx.data::<DataLoader<BenchmarkLoader>>()?;
        loader
            .load_one(self.benchmark_id)
            .await?
            .ok_or_else(|| "Benchmark not found".into())
    }

    async fn measure(&self, ctx: &Context<'_>) -> Result<super::Measure> {
        let loader = ctx.data::<DataLoader<MeasureLoader>>()?;
        loader
            .load_one(self.measure_id)
            .await?
            .ok_or_else(|| "Measure not found".into())
    }
}
