use async_graphql::dataloader::Loader;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::collections::HashMap;
use uuid::Uuid;

use crate::entities::{self, branch, testbed, benchmark, measure, metric, threshold};
use crate::graphql::types::{Branch, Testbed, Benchmark, Measure, Metric, Threshold};

pub struct BranchLoader {
    pub db: DatabaseConnection,
}

impl Loader<Uuid> for BranchLoader {
    type Value = Branch;
    type Error = async_graphql::Error;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let branches = entities::Branch::find()
            .filter(branch::Column::Id.is_in(keys.to_vec()))
            .all(&self.db)
            .await?;

        Ok(branches.into_iter().map(|b| (b.id, b.into())).collect())
    }
}

pub struct TestbedLoader {
    pub db: DatabaseConnection,
}

impl Loader<Uuid> for TestbedLoader {
    type Value = Testbed;
    type Error = async_graphql::Error;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let testbeds = entities::Testbed::find()
            .filter(testbed::Column::Id.is_in(keys.to_vec()))
            .all(&self.db)
            .await?;

        Ok(testbeds.into_iter().map(|t| (t.id, t.into())).collect())
    }
}

pub struct BenchmarkLoader {
    pub db: DatabaseConnection,
}

impl Loader<Uuid> for BenchmarkLoader {
    type Value = Benchmark;
    type Error = async_graphql::Error;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let benchmarks = entities::Benchmark::find()
            .filter(benchmark::Column::Id.is_in(keys.to_vec()))
            .all(&self.db)
            .await?;

        Ok(benchmarks.into_iter().map(|b| (b.id, b.into())).collect())
    }
}

pub struct MeasureLoader {
    pub db: DatabaseConnection,
}

impl Loader<Uuid> for MeasureLoader {
    type Value = Measure;
    type Error = async_graphql::Error;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let measures = entities::Measure::find()
            .filter(measure::Column::Id.is_in(keys.to_vec()))
            .all(&self.db)
            .await?;

        Ok(measures.into_iter().map(|m| (m.id, m.into())).collect())
    }
}

pub struct MetricLoader {
    pub db: DatabaseConnection,
}

impl Loader<Uuid> for MetricLoader {
    type Value = Metric;
    type Error = async_graphql::Error;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let metrics = entities::Metric::find()
            .filter(metric::Column::Id.is_in(keys.to_vec()))
            .all(&self.db)
            .await?;

        Ok(metrics.into_iter().map(|m| (m.id, m.into())).collect())
    }
}

pub struct ThresholdLoader {
    pub db: DatabaseConnection,
}

impl Loader<Uuid> for ThresholdLoader {
    type Value = Threshold;
    type Error = async_graphql::Error;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let thresholds = entities::Threshold::find()
            .filter(threshold::Column::Id.is_in(keys.to_vec()))
            .all(&self.db)
            .await?;

        Ok(thresholds.into_iter().map(|t| (t.id, t.into())).collect())
    }
}
