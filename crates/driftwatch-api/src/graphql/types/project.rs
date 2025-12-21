use async_graphql::{ComplexObject, Context, InputObject, Result, SimpleObject, ID};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{
    self, alert, benchmark, branch, measure, project, report, testbed, threshold,
};

#[derive(SimpleObject, Serialize, Deserialize)]
#[graphql(complex, cache_control(max_age = 300))]
pub struct Project {
    pub id: ID,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub public: bool,
    pub github_repo: Option<String>,
    pub github_pr_comments: bool,
    pub github_status_checks: bool,
    pub has_github_token: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<project::Model> for Project {
    fn from(model: project::Model) -> Self {
        Self {
            id: ID(model.id.to_string()),
            slug: model.slug,
            name: model.name,
            description: model.description,
            public: model.public,
            github_repo: model.github_repo,
            github_pr_comments: model.github_pr_comments,
            github_status_checks: model.github_status_checks,
            has_github_token: model.github_token.is_some(),
            created_at: model.created_at.into(),
            updated_at: model.updated_at.into(),
        }
    }
}

#[ComplexObject]
impl Project {
    async fn branches(&self, ctx: &Context<'_>) -> Result<Vec<super::Branch>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let project_id = Uuid::parse_str(&self.id.0)?;

        let branches = entities::Branch::find()
            .filter(branch::Column::ProjectId.eq(project_id))
            .order_by_asc(branch::Column::Name)
            .all(db)
            .await?;

        Ok(branches.into_iter().map(Into::into).collect())
    }

    async fn testbeds(&self, ctx: &Context<'_>) -> Result<Vec<super::Testbed>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let project_id = Uuid::parse_str(&self.id.0)?;

        let testbeds = entities::Testbed::find()
            .filter(testbed::Column::ProjectId.eq(project_id))
            .order_by_asc(testbed::Column::Name)
            .all(db)
            .await?;

        Ok(testbeds.into_iter().map(Into::into).collect())
    }

    async fn measures(&self, ctx: &Context<'_>) -> Result<Vec<super::Measure>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let project_id = Uuid::parse_str(&self.id.0)?;

        let measures = entities::Measure::find()
            .filter(measure::Column::ProjectId.eq(project_id))
            .order_by_asc(measure::Column::Name)
            .all(db)
            .await?;

        Ok(measures.into_iter().map(Into::into).collect())
    }

    async fn benchmarks(&self, ctx: &Context<'_>) -> Result<Vec<super::Benchmark>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let project_id = Uuid::parse_str(&self.id.0)?;

        let benchmarks = entities::Benchmark::find()
            .filter(benchmark::Column::ProjectId.eq(project_id))
            .order_by_asc(benchmark::Column::Name)
            .all(db)
            .await?;

        Ok(benchmarks.into_iter().map(Into::into).collect())
    }

    async fn reports(&self, ctx: &Context<'_>, limit: Option<i32>) -> Result<Vec<super::Report>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let project_id = Uuid::parse_str(&self.id.0)?;

        let mut query = entities::Report::find()
            .filter(report::Column::ProjectId.eq(project_id))
            .order_by_desc(report::Column::CreatedAt);

        if let Some(limit) = limit {
            query = query.limit(limit as u64);
        }

        let reports = query.all(db).await?;
        Ok(reports.into_iter().map(Into::into).collect())
    }

    async fn thresholds(&self, ctx: &Context<'_>) -> Result<Vec<super::Threshold>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let project_id = Uuid::parse_str(&self.id.0)?;

        let thresholds = entities::Threshold::find()
            .filter(threshold::Column::ProjectId.eq(project_id))
            .all(db)
            .await?;

        Ok(thresholds.into_iter().map(Into::into).collect())
    }

    async fn alerts(
        &self,
        ctx: &Context<'_>,
        status: Option<super::AlertStatusInput>,
    ) -> Result<Vec<super::Alert>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let project_id = Uuid::parse_str(&self.id.0)?;

        // Get all thresholds for this project
        let threshold_ids: Vec<Uuid> = entities::Threshold::find()
            .filter(threshold::Column::ProjectId.eq(project_id))
            .all(db)
            .await?
            .into_iter()
            .map(|t| t.id)
            .collect();

        let mut query =
            entities::Alert::find().filter(alert::Column::ThresholdId.is_in(threshold_ids));

        if let Some(status) = status {
            query = query.filter(alert::Column::Status.eq(status.to_db_value()));
        }

        let alerts = query
            .order_by_desc(alert::Column::CreatedAt)
            .all(db)
            .await?;
        Ok(alerts.into_iter().map(Into::into).collect())
    }
}

#[derive(InputObject)]
pub struct CreateProjectInput {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateProjectInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub public: Option<bool>,
}

#[derive(InputObject)]
pub struct GitHubSettingsInput {
    pub github_repo: Option<String>,
    pub github_token: Option<String>,
    pub github_pr_comments: Option<bool>,
    pub github_status_checks: Option<bool>,
}
