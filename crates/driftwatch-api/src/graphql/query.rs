use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use tracing::{info_span, instrument, Instrument};

use super::types::{ApiKey, Project, User};
use crate::auth::AuthUser;
use crate::cache::AppCache;
use crate::entities::{self, project};
use crate::grpc::AuthServiceImpl;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    #[instrument(skip(self, ctx), fields(user_id))]
    async fn projects(&self, ctx: &Context<'_>) -> Result<Vec<Project>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let user = ctx.data::<AuthUser>()?;
        let cache = ctx.data::<AppCache>()?;
        let user_id = user.user_id();

        tracing::Span::current().record("user_id", user_id.to_string());

        let cache_key = format!("user:{}:projects", user_id);

        let cache_result = async { cache.projects.get(&cache_key).await }
            .instrument(info_span!("cache_check"))
            .await;

        if let Some(cached) = cache_result {
            let deserialize_result = async { serde_json::from_str::<Vec<Project>>(&cached) }
                .instrument(info_span!("deserialize"))
                .await;

            if let Ok(projects) = deserialize_result {
                tracing::info!(cache = "hit", count = projects.len());
                return Ok(projects);
            }
        }

        tracing::info!(cache = "miss");

        let db_result = async {
            entities::Project::find()
                .filter(project::Column::UserId.eq(user_id))
                .order_by_desc(project::Column::CreatedAt)
                .all(db)
                .await
        }
        .instrument(info_span!("db_query", table = "project"))
        .await?;

        let projects: Vec<Project> = db_result.into_iter().map(Into::into).collect();

        tracing::info!(count = projects.len());

        if let Ok(json) = serde_json::to_string(&projects) {
            async {
                cache.projects.insert(cache_key, json).await;
            }
            .instrument(info_span!("cache_insert"))
            .await;
        }

        Ok(projects)
    }

    #[instrument(skip(self, ctx), fields(user_id))]
    async fn project(&self, ctx: &Context<'_>, slug: String) -> Result<Option<Project>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let user = ctx.data::<AuthUser>()?;
        let cache = ctx.data::<AppCache>()?;
        let user_id = user.user_id();

        tracing::Span::current().record("user_id", user_id.to_string());

        let cache_key = format!("user:{}:project:{}", user_id, slug);

        let cache_result = async { cache.project.get(&cache_key).await }
            .instrument(info_span!("cache_check"))
            .await;

        if let Some(cached) = cache_result {
            let deserialize_result = async { serde_json::from_str::<Project>(&cached) }
                .instrument(info_span!("deserialize"))
                .await;

            if let Ok(project) = deserialize_result {
                tracing::info!(cache = "hit");
                return Ok(Some(project));
            }
        }

        tracing::info!(cache = "miss");

        let project = async {
            entities::Project::find()
                .filter(project::Column::UserId.eq(user_id))
                .filter(project::Column::Slug.eq(&slug))
                .one(db)
                .await
        }
        .instrument(info_span!("db_query", table = "project"))
        .await?;

        let result = match project {
            Some(p) => {
                let project_gql: Project = p.into();
                if let Ok(json) = serde_json::to_string(&project_gql) {
                    async {
                        cache.project.insert(cache_key, json).await;
                    }
                    .instrument(info_span!("cache_insert"))
                    .await;
                }
                tracing::info!(found = true);
                Some(project_gql)
            }
            None => {
                tracing::info!(found = false);
                None
            }
        };

        Ok(result)
    }

    async fn me(&self, ctx: &Context<'_>) -> Result<User> {
        let user = ctx.data::<AuthUser>()?;
        Ok(user.user.clone().into())
    }

    async fn api_keys(&self, ctx: &Context<'_>) -> Result<Vec<ApiKey>> {
        let auth_service = ctx.data::<Arc<AuthServiceImpl>>()?;
        let user = ctx.data::<AuthUser>()?;

        if !user.is_session_auth() {
            return Err("Listing API keys requires session authentication, not API key".into());
        }

        let api_keys = auth_service
            .list_api_keys_direct(&user.token)
            .await
            .map_err(async_graphql::Error::new)?;

        Ok(api_keys.into_iter().map(Into::into).collect())
    }
}
