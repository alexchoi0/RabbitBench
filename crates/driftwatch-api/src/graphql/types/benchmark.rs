use async_graphql::{SimpleObject, ID};

use crate::entities::benchmark;

#[derive(SimpleObject, Clone)]
#[graphql(cache_control(max_age = 300))]
pub struct Benchmark {
    pub id: ID,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<benchmark::Model> for Benchmark {
    fn from(model: benchmark::Model) -> Self {
        Self {
            id: ID(model.id.to_string()),
            name: model.name,
            created_at: model.created_at.into(),
        }
    }
}
