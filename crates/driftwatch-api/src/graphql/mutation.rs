use async_graphql::{Context, Object, Result, ID};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use super::types::{
    CreateProjectInput, CreateThresholdInput, GitHubSettingsInput, Project, Threshold,
    UpdateProjectInput,
};
use crate::auth::AuthUser;
use crate::cache::AppCache;
use crate::entities::{self, measure, project, threshold};

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_project(
        &self,
        ctx: &Context<'_>,
        input: CreateProjectInput,
    ) -> Result<Project> {
        let db = ctx.data::<DatabaseConnection>()?;
        let user = ctx.data::<AuthUser>()?;
        let cache = ctx.data::<AppCache>()?;
        let user_id = user.user_id();

        let existing = entities::Project::find()
            .filter(project::Column::UserId.eq(user_id))
            .filter(project::Column::Slug.eq(&input.slug))
            .one(db)
            .await?;

        if existing.is_some() {
            return Err("A workspace with this slug already exists".into());
        }

        let now = Utc::now().fixed_offset();
        let project = project::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            slug: Set(input.slug),
            name: Set(input.name),
            description: Set(input.description),
            public: Set(false),
            github_repo: Set(None),
            github_token: Set(None),
            github_pr_comments: Set(false),
            github_status_checks: Set(false),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let project = project.insert(db).await?;

        let measure = measure::ActiveModel {
            id: Set(Uuid::new_v4()),
            project_id: Set(project.id),
            name: Set("latency".to_string()),
            units: Set(Some("ns".to_string())),
            created_at: Set(now),
            updated_at: Set(now),
        };
        measure.insert(db).await?;

        cache.invalidate_user_projects(user_id).await;

        Ok(project.into())
    }

    async fn update_project(
        &self,
        ctx: &Context<'_>,
        slug: String,
        input: UpdateProjectInput,
    ) -> Result<Project> {
        let db = ctx.data::<DatabaseConnection>()?;
        let user = ctx.data::<AuthUser>()?;
        let cache = ctx.data::<AppCache>()?;
        let user_id = user.user_id();

        let project = entities::Project::find()
            .filter(project::Column::UserId.eq(user_id))
            .filter(project::Column::Slug.eq(&slug))
            .one(db)
            .await?
            .ok_or("Workspace not found")?;

        let mut active: project::ActiveModel = project.into();

        if let Some(name) = input.name {
            active.name = Set(name);
        }
        if let Some(description) = input.description {
            active.description = Set(Some(description));
        }
        if let Some(public) = input.public {
            active.public = Set(public);
        }
        active.updated_at = Set(Utc::now().fixed_offset());

        let updated = active.update(db).await?;

        cache.invalidate_project(user_id, &slug).await;

        Ok(updated.into())
    }

    async fn delete_project(&self, ctx: &Context<'_>, slug: String) -> Result<bool> {
        let db = ctx.data::<DatabaseConnection>()?;
        let user = ctx.data::<AuthUser>()?;
        let cache = ctx.data::<AppCache>()?;
        let user_id = user.user_id();

        let project = entities::Project::find()
            .filter(project::Column::UserId.eq(user_id))
            .filter(project::Column::Slug.eq(&slug))
            .one(db)
            .await?
            .ok_or("Workspace not found")?;

        entities::Project::delete_by_id(project.id).exec(db).await?;

        cache.invalidate_project(user_id, &slug).await;

        Ok(true)
    }

    async fn update_github_settings(
        &self,
        ctx: &Context<'_>,
        slug: String,
        input: GitHubSettingsInput,
    ) -> Result<Project> {
        let db = ctx.data::<DatabaseConnection>()?;
        let user = ctx.data::<AuthUser>()?;
        let cache = ctx.data::<AppCache>()?;
        let user_id = user.user_id();

        let project = entities::Project::find()
            .filter(project::Column::UserId.eq(user_id))
            .filter(project::Column::Slug.eq(&slug))
            .one(db)
            .await?
            .ok_or("Workspace not found")?;

        let mut active: project::ActiveModel = project.into();

        if let Some(repo) = input.github_repo {
            active.github_repo = Set(if repo.is_empty() { None } else { Some(repo) });
        }
        if let Some(token) = input.github_token {
            if !token.is_empty() {
                active.github_token = Set(Some(token));
            }
        }
        if let Some(pr_comments) = input.github_pr_comments {
            active.github_pr_comments = Set(pr_comments);
        }
        if let Some(status_checks) = input.github_status_checks {
            active.github_status_checks = Set(status_checks);
        }
        active.updated_at = Set(Utc::now().fixed_offset());

        let updated = active.update(db).await?;

        cache.invalidate_project(user_id, &slug).await;

        Ok(updated.into())
    }

    async fn create_threshold(
        &self,
        ctx: &Context<'_>,
        input: CreateThresholdInput,
    ) -> Result<Threshold> {
        let db = ctx.data::<DatabaseConnection>()?;
        let user = ctx.data::<AuthUser>()?;
        let cache = ctx.data::<AppCache>()?;
        let user_id = user.user_id();

        let project = entities::Project::find()
            .filter(project::Column::UserId.eq(user_id))
            .filter(project::Column::Slug.eq(&input.project_slug))
            .one(db)
            .await?
            .ok_or("Workspace not found")?;

        let project_slug = project.slug.clone();
        let measure_id = Uuid::parse_str(&input.measure_id.0)?;
        let branch_id = input
            .branch_id
            .map(|id| Uuid::parse_str(&id.0))
            .transpose()?;
        let testbed_id = input
            .testbed_id
            .map(|id| Uuid::parse_str(&id.0))
            .transpose()?;

        let now = Utc::now().fixed_offset();
        let threshold = threshold::ActiveModel {
            id: Set(Uuid::new_v4()),
            project_id: Set(project.id),
            measure_id: Set(measure_id),
            branch_id: Set(branch_id),
            testbed_id: Set(testbed_id),
            upper_boundary: Set(input.upper_boundary),
            lower_boundary: Set(input.lower_boundary),
            min_sample_size: Set(input.min_sample_size.unwrap_or(2)),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let threshold = threshold.insert(db).await?;

        cache.invalidate_project(user_id, &project_slug).await;

        Ok(threshold.into())
    }

    async fn delete_threshold(&self, ctx: &Context<'_>, id: ID) -> Result<bool> {
        let db = ctx.data::<DatabaseConnection>()?;
        let user = ctx.data::<AuthUser>()?;
        let cache = ctx.data::<AppCache>()?;
        let user_id = user.user_id();

        let threshold_id = Uuid::parse_str(&id.0)?;

        let threshold = entities::Threshold::find_by_id(threshold_id)
            .one(db)
            .await?
            .ok_or("Threshold not found")?;

        let project = entities::Project::find_by_id(threshold.project_id)
            .one(db)
            .await?
            .ok_or("Project not found")?;

        if project.user_id != user_id {
            return Err("Unauthorized".into());
        }

        let project_slug = project.slug.clone();
        entities::Threshold::delete_by_id(threshold_id)
            .exec(db)
            .await?;

        cache.invalidate_project(user_id, &project_slug).await;

        Ok(true)
    }
}
