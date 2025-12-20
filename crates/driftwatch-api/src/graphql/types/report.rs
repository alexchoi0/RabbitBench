use async_graphql::dataloader::DataLoader;
use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use uuid::Uuid;

use crate::loaders::{BranchLoader, TestbedLoader};

#[derive(SimpleObject)]
#[graphql(complex, cache_control(max_age = 3600))]
pub struct Report {
    pub id: ID,
    pub git_hash: Option<String>,
    pub pr_number: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[graphql(skip)]
    pub branch_id: Uuid,
    #[graphql(skip)]
    pub testbed_id: Uuid,
}

impl From<crate::entities::report::Model> for Report {
    fn from(model: crate::entities::report::Model) -> Self {
        Self {
            id: ID(model.id.to_string()),
            git_hash: model.git_hash,
            pr_number: model.pr_number,
            created_at: model.created_at.into(),
            branch_id: model.branch_id,
            testbed_id: model.testbed_id,
        }
    }
}

#[ComplexObject]
impl Report {
    async fn branch(&self, ctx: &Context<'_>) -> Result<super::Branch> {
        let loader = ctx.data::<DataLoader<BranchLoader>>()?;
        loader
            .load_one(self.branch_id)
            .await?
            .ok_or_else(|| "Branch not found".into())
    }

    async fn testbed(&self, ctx: &Context<'_>) -> Result<super::Testbed> {
        let loader = ctx.data::<DataLoader<TestbedLoader>>()?;
        loader
            .load_one(self.testbed_id)
            .await?
            .ok_or_else(|| "Testbed not found".into())
    }
}
