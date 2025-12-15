use async_graphql::{SimpleObject, InputObject, ID};

use crate::entities::threshold;

#[derive(SimpleObject, Clone)]
#[graphql(cache_control(max_age = 300))]
pub struct Threshold {
    pub id: ID,
    pub measure_id: ID,
    pub branch_id: Option<ID>,
    pub testbed_id: Option<ID>,
    pub upper_boundary: Option<f64>,
    pub lower_boundary: Option<f64>,
    pub min_sample_size: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<threshold::Model> for Threshold {
    fn from(model: threshold::Model) -> Self {
        Self {
            id: ID(model.id.to_string()),
            measure_id: ID(model.measure_id.to_string()),
            branch_id: model.branch_id.map(|id| ID(id.to_string())),
            testbed_id: model.testbed_id.map(|id| ID(id.to_string())),
            upper_boundary: model.upper_boundary,
            lower_boundary: model.lower_boundary,
            min_sample_size: model.min_sample_size,
            created_at: model.created_at.into(),
        }
    }
}

#[derive(InputObject)]
pub struct CreateThresholdInput {
    pub project_slug: String,
    pub measure_id: ID,
    pub branch_id: Option<ID>,
    pub testbed_id: Option<ID>,
    pub upper_boundary: Option<f64>,
    pub lower_boundary: Option<f64>,
    pub min_sample_size: Option<i32>,
}
