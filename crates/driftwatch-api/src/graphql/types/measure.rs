use async_graphql::{SimpleObject, ID};

use crate::entities::measure;

#[derive(SimpleObject, Clone)]
#[graphql(cache_control(max_age = 300))]
pub struct Measure {
    pub id: ID,
    pub name: String,
    pub units: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<measure::Model> for Measure {
    fn from(model: measure::Model) -> Self {
        Self {
            id: ID(model.id.to_string()),
            name: model.name,
            units: model.units,
            created_at: model.created_at.into(),
        }
    }
}
