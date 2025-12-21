use async_graphql::dataloader::Loader;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::collections::HashMap;
use uuid::Uuid;

use crate::entities::{self, benchmark, branch, measure, metric, testbed, threshold};
use crate::graphql::types::{Benchmark, Branch, Measure, Metric, Testbed, Threshold};

macro_rules! define_loader {
    ($name:ident, $entity:ty, $column:expr, $output:ty) => {
        pub struct $name {
            pub db: DatabaseConnection,
        }

        impl Loader<Uuid> for $name {
            type Value = $output;
            type Error = async_graphql::Error;

            async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
                let items = <$entity>::find()
                    .filter($column.is_in(keys.to_vec()))
                    .all(&self.db)
                    .await?;

                Ok(items
                    .into_iter()
                    .map(|item| (item.id, item.into()))
                    .collect())
            }
        }
    };
}

define_loader!(BranchLoader, entities::Branch, branch::Column::Id, Branch);
define_loader!(
    TestbedLoader,
    entities::Testbed,
    testbed::Column::Id,
    Testbed
);
define_loader!(
    BenchmarkLoader,
    entities::Benchmark,
    benchmark::Column::Id,
    Benchmark
);
define_loader!(
    MeasureLoader,
    entities::Measure,
    measure::Column::Id,
    Measure
);
define_loader!(MetricLoader, entities::Metric, metric::Column::Id, Metric);
define_loader!(
    ThresholdLoader,
    entities::Threshold,
    threshold::Column::Id,
    Threshold
);
