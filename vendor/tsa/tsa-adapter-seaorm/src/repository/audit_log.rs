use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};
use std::sync::Arc;
use tsa_core::{AuditAction, AuditLog, AuditLogRepository, Result, TsaError};
use uuid::Uuid;

use crate::entity::audit_log::{ActiveModel, Column, Entity};

#[derive(Clone)]
pub struct SeaOrmAuditLogRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmAuditLogRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

impl From<crate::entity::audit_log::Model> for AuditLog {
    fn from(model: crate::entity::audit_log::Model) -> Self {
        let action: AuditAction = serde_json::from_value(serde_json::json!(model.action))
            .unwrap_or(AuditAction::SigninFailed);
        AuditLog {
            id: model.id,
            user_id: model.user_id,
            actor_id: model.actor_id,
            action,
            ip_address: model.ip_address,
            user_agent: model.user_agent,
            resource_type: model.resource_type,
            resource_id: model.resource_id,
            details: model.details,
            success: model.success,
            error_message: model.error_message,
            created_at: model.created_at,
        }
    }
}

#[async_trait]
#[allow(clippy::needless_borrows_for_generic_args)]
impl AuditLogRepository for SeaOrmAuditLogRepository {
    async fn create(&self, log: &AuditLog) -> Result<AuditLog> {
        let action_str =
            serde_json::to_value(&log.action).map_err(|e| TsaError::Internal(e.to_string()))?;
        let action_str = action_str.as_str().unwrap_or("other").to_string();

        let active_model = ActiveModel {
            id: Set(log.id),
            user_id: Set(log.user_id),
            actor_id: Set(log.actor_id),
            action: Set(action_str),
            ip_address: Set(log.ip_address.clone()),
            user_agent: Set(log.user_agent.clone()),
            resource_type: Set(log.resource_type.clone()),
            resource_id: Set(log.resource_id.clone()),
            details: Set(log.details.clone()),
            success: Set(log.success),
            error_message: Set(log.error_message.clone()),
            created_at: Set(log.created_at),
        };

        let result = active_model
            .insert(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.into())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<AuditLog>> {
        let result = Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn find_by_user(&self, user_id: Uuid, limit: u32, offset: u32) -> Result<Vec<AuditLog>> {
        let results = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .order_by_desc(Column::CreatedAt)
            .offset(offset as u64)
            .limit(limit as u64)
            .all(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn find_by_action(
        &self,
        action: AuditAction,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<AuditLog>> {
        let action_str =
            serde_json::to_value(action).map_err(|e| TsaError::Internal(e.to_string()))?;
        let action_str = action_str.as_str().unwrap_or("other").to_string();

        let results = Entity::find()
            .filter(Column::Action.eq(action_str))
            .order_by_desc(Column::CreatedAt)
            .offset(offset as u64)
            .limit(limit as u64)
            .all(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn find_by_ip(&self, ip_address: &str, limit: u32, offset: u32) -> Result<Vec<AuditLog>> {
        let results = Entity::find()
            .filter(Column::IpAddress.eq(ip_address))
            .order_by_desc(Column::CreatedAt)
            .offset(offset as u64)
            .limit(limit as u64)
            .all(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn find_recent(&self, limit: u32, offset: u32) -> Result<Vec<AuditLog>> {
        let results = Entity::find()
            .order_by_desc(Column::CreatedAt)
            .offset(offset as u64)
            .limit(limit as u64)
            .all(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn find_failed(&self, limit: u32, offset: u32) -> Result<Vec<AuditLog>> {
        let results = Entity::find()
            .filter(Column::Success.eq(false))
            .order_by_desc(Column::CreatedAt)
            .offset(offset as u64)
            .limit(limit as u64)
            .all(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn count_by_user(&self, user_id: Uuid) -> Result<u64> {
        let count = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .count(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(count)
    }

    async fn count_failed_by_user_since(
        &self,
        user_id: Uuid,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<u32> {
        let count = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::Success.eq(false))
            .filter(Column::CreatedAt.gte(since))
            .count(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(count as u32)
    }

    async fn delete_older_than(&self, before: chrono::DateTime<chrono::Utc>) -> Result<u64> {
        let result = Entity::delete_many()
            .filter(Column::CreatedAt.lt(before))
            .exec(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.rows_affected)
    }
}
