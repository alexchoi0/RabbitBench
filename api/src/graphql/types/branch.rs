use async_graphql::{SimpleObject, ID};

use crate::entities::branch;

#[derive(SimpleObject, Clone)]
#[graphql(cache_control(max_age = 300))]
pub struct Branch {
    pub id: ID,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<branch::Model> for Branch {
    fn from(model: branch::Model) -> Self {
        Self {
            id: ID(model.id.to_string()),
            name: model.name,
            created_at: model.created_at.into(),
        }
    }
}
