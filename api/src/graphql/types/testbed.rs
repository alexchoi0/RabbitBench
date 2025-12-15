use async_graphql::{SimpleObject, ID};

use crate::entities::testbed;

#[derive(SimpleObject, Clone)]
#[graphql(cache_control(max_age = 300))]
pub struct Testbed {
    pub id: ID,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<testbed::Model> for Testbed {
    fn from(model: testbed::Model) -> Self {
        Self {
            id: ID(model.id.to_string()),
            name: model.name,
            created_at: model.created_at.into(),
        }
    }
}
