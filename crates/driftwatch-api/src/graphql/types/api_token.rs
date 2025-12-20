use async_graphql::{SimpleObject, ID};
use serde::{Deserialize, Serialize};

use crate::entities::api_token;

#[derive(SimpleObject, Serialize, Deserialize)]
#[graphql(cache_control(max_age = 60))]
pub struct ApiToken {
    pub id: ID,
    pub name: String,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<api_token::Model> for ApiToken {
    fn from(model: api_token::Model) -> Self {
        Self {
            id: ID(model.id.to_string()),
            name: model.name,
            last_used_at: model.last_used_at.map(|dt| dt.into()),
            created_at: model.created_at.into(),
        }
    }
}

#[derive(SimpleObject)]
pub struct CreateApiTokenResult {
    pub token: ApiToken,
    pub secret: String,
}
