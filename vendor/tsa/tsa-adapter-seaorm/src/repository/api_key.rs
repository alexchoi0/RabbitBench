use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;
use tsa_core::{ApiKey, ApiKeyRepository, Result, TsaError};
use uuid::Uuid;

use crate::entity::api_key::{ActiveModel, Column, Entity};

#[derive(Clone)]
pub struct SeaOrmApiKeyRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmApiKeyRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

impl From<crate::entity::api_key::Model> for ApiKey {
    fn from(model: crate::entity::api_key::Model) -> Self {
        let scopes: Vec<String> = serde_json::from_value(model.scopes).unwrap_or_default();
        ApiKey {
            id: model.id,
            user_id: model.user_id,
            organization_id: model.organization_id,
            name: model.name,
            key_hash: model.key_hash,
            prefix: model.prefix,
            scopes,
            expires_at: model.expires_at,
            last_used_at: model.last_used_at,
            created_at: model.created_at,
        }
    }
}

#[async_trait]
impl ApiKeyRepository for SeaOrmApiKeyRepository {
    async fn create(&self, api_key: &ApiKey) -> Result<ApiKey> {
        let scopes_json =
            serde_json::to_value(&api_key.scopes).map_err(|e| TsaError::Internal(e.to_string()))?;

        let active_model = ActiveModel {
            id: Set(api_key.id),
            user_id: Set(api_key.user_id),
            organization_id: Set(api_key.organization_id),
            name: Set(api_key.name.clone()),
            key_hash: Set(api_key.key_hash.clone()),
            prefix: Set(api_key.prefix.clone()),
            scopes: Set(scopes_json),
            expires_at: Set(api_key.expires_at),
            last_used_at: Set(api_key.last_used_at),
            created_at: Set(api_key.created_at),
        };

        let result = active_model
            .insert(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.into())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<ApiKey>> {
        let result = Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn find_by_key_hash(&self, key_hash: &str) -> Result<Option<ApiKey>> {
        let result = Entity::find()
            .filter(Column::KeyHash.eq(key_hash))
            .one(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn find_by_prefix(&self, prefix: &str) -> Result<Option<ApiKey>> {
        let result = Entity::find()
            .filter(Column::Prefix.eq(prefix))
            .one(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<ApiKey>> {
        let results = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .all(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn find_by_organization(&self, organization_id: Uuid) -> Result<Vec<ApiKey>> {
        let results = Entity::find()
            .filter(Column::OrganizationId.eq(Some(organization_id)))
            .all(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn update(&self, api_key: &ApiKey) -> Result<ApiKey> {
        let scopes_json =
            serde_json::to_value(&api_key.scopes).map_err(|e| TsaError::Internal(e.to_string()))?;

        let active_model = ActiveModel {
            id: Set(api_key.id),
            user_id: Set(api_key.user_id),
            organization_id: Set(api_key.organization_id),
            name: Set(api_key.name.clone()),
            key_hash: Set(api_key.key_hash.clone()),
            prefix: Set(api_key.prefix.clone()),
            scopes: Set(scopes_json),
            expires_at: Set(api_key.expires_at),
            last_used_at: Set(api_key.last_used_at),
            created_at: Set(api_key.created_at),
        };

        let result = active_model
            .update(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.into())
    }

    async fn update_last_used(&self, id: Uuid) -> Result<()> {
        use sea_orm::ActiveValue::NotSet;

        let active_model = ActiveModel {
            id: Set(id),
            last_used_at: Set(Some(Utc::now())),
            user_id: NotSet,
            organization_id: NotSet,
            name: NotSet,
            key_hash: NotSet,
            prefix: NotSet,
            scopes: NotSet,
            expires_at: NotSet,
            created_at: NotSet,
        };

        active_model
            .update(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        Entity::delete_by_id(id)
            .exec(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(())
    }

    async fn delete_by_user(&self, user_id: Uuid) -> Result<()> {
        Entity::delete_many()
            .filter(Column::UserId.eq(user_id))
            .exec(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(())
    }
}
