use async_graphql::{Context, Object, Result};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, QueryOrder};
use tracing::{instrument, info_span, Instrument};

use crate::auth::AuthUser;
use crate::cache::AppCache;
use crate::entities::{self, project, api_token};
use super::types::{Project, ApiToken};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    #[instrument(skip(self, ctx), fields(user_id))]
    async fn projects(&self, ctx: &Context<'_>) -> Result<Vec<Project>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let user = ctx.data::<AuthUser>()?;
        let cache = ctx.data::<AppCache>()?;

        tracing::Span::current().record("user_id", &user.user_id);

        let cache_key = format!("user:{}:projects", user.user_id);

        // Check cache first
        let cache_result = async {
            cache.projects.get(&cache_key).await
        }
        .instrument(info_span!("cache_check"))
        .await;

        if let Some(cached) = cache_result {
            let deserialize_result = async {
                serde_json::from_str::<Vec<Project>>(&cached)
            }
            .instrument(info_span!("deserialize"))
            .await;

            if let Ok(projects) = deserialize_result {
                tracing::info!(cache = "hit", count = projects.len());
                return Ok(projects);
            }
        }

        tracing::info!(cache = "miss");

        // Cache miss - query database
        let db_result = async {
            entities::Project::find()
                .filter(project::Column::UserId.eq(&user.user_id))
                .order_by_desc(project::Column::CreatedAt)
                .all(db)
                .await
        }
        .instrument(info_span!("db_query", table = "project"))
        .await?;

        let projects: Vec<Project> = db_result.into_iter().map(Into::into).collect();

        tracing::info!(count = projects.len());

        // Store in cache
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

        tracing::Span::current().record("user_id", &user.user_id);

        let cache_key = format!("user:{}:project:{}", user.user_id, slug);

        // Check cache first
        let cache_result = async {
            cache.project.get(&cache_key).await
        }
        .instrument(info_span!("cache_check"))
        .await;

        if let Some(cached) = cache_result {
            let deserialize_result = async {
                serde_json::from_str::<Project>(&cached)
            }
            .instrument(info_span!("deserialize"))
            .await;

            if let Ok(project) = deserialize_result {
                tracing::info!(cache = "hit");
                return Ok(Some(project));
            }
        }

        tracing::info!(cache = "miss");

        // Cache miss - query database
        let project = async {
            entities::Project::find()
                .filter(project::Column::UserId.eq(&user.user_id))
                .filter(project::Column::Slug.eq(&slug))
                .one(db)
                .await
        }
        .instrument(info_span!("db_query", table = "project"))
        .await?;

        // Store in cache if found
        if let Some(ref p) = project {
            let project_gql: Project = p.clone().into();
            if let Ok(json) = serde_json::to_string(&project_gql) {
                async {
                    cache.project.insert(cache_key, json).await;
                }
                .instrument(info_span!("cache_insert"))
                .await;
            }
        }

        tracing::info!(found = project.is_some());

        Ok(project.map(Into::into))
    }

    #[instrument(skip(self, ctx), fields(user_id))]
    async fn api_tokens(&self, ctx: &Context<'_>) -> Result<Vec<ApiToken>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let user = ctx.data::<AuthUser>()?;
        let cache = ctx.data::<AppCache>()?;

        tracing::Span::current().record("user_id", &user.user_id);

        let cache_key = format!("user:{}:tokens", user.user_id);

        // Check cache first
        let cache_result = async {
            cache.tokens.get(&cache_key).await
        }
        .instrument(info_span!("cache_check"))
        .await;

        if let Some(cached) = cache_result {
            let deserialize_result = async {
                serde_json::from_str::<Vec<ApiToken>>(&cached)
            }
            .instrument(info_span!("deserialize"))
            .await;

            if let Ok(tokens) = deserialize_result {
                tracing::info!(cache = "hit", count = tokens.len());
                return Ok(tokens);
            }
        }

        tracing::info!(cache = "miss");

        // Cache miss - query database
        let db_result = async {
            entities::ApiToken::find()
                .filter(api_token::Column::UserId.eq(&user.user_id))
                .order_by_desc(api_token::Column::CreatedAt)
                .all(db)
                .await
        }
        .instrument(info_span!("db_query", table = "api_token"))
        .await?;

        let tokens: Vec<ApiToken> = db_result.into_iter().map(Into::into).collect();

        tracing::info!(count = tokens.len());

        // Store in cache
        if let Ok(json) = serde_json::to_string(&tokens) {
            async {
                cache.tokens.insert(cache_key, json).await;
            }
            .instrument(info_span!("cache_insert"))
            .await;
        }

        Ok(tokens)
    }
}
