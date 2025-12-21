use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;
use tsa_core::{ImpersonationSession, ImpersonationSessionRepository, Result, TsaError};
use uuid::Uuid;

use crate::entity::impersonation_session::{ActiveModel, Column, Entity};

#[derive(Clone)]
pub struct SeaOrmImpersonationSessionRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmImpersonationSessionRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

impl From<crate::entity::impersonation_session::Model> for ImpersonationSession {
    fn from(model: crate::entity::impersonation_session::Model) -> Self {
        ImpersonationSession {
            id: model.id,
            admin_id: model.admin_id,
            target_user_id: model.target_user_id,
            original_session_id: model.original_session_id,
            impersonation_session_id: model.impersonation_session_id,
            reason: model.reason,
            started_at: model.started_at,
            ended_at: model.ended_at,
        }
    }
}

#[async_trait]
impl ImpersonationSessionRepository for SeaOrmImpersonationSessionRepository {
    async fn create(&self, session: &ImpersonationSession) -> Result<ImpersonationSession> {
        let active_model = ActiveModel {
            id: Set(session.id),
            admin_id: Set(session.admin_id),
            target_user_id: Set(session.target_user_id),
            original_session_id: Set(session.original_session_id),
            impersonation_session_id: Set(session.impersonation_session_id),
            reason: Set(session.reason.clone()),
            started_at: Set(session.started_at),
            ended_at: Set(session.ended_at),
        };

        let result = active_model
            .insert(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.into())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<ImpersonationSession>> {
        let result = Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn find_by_session_id(&self, session_id: Uuid) -> Result<Option<ImpersonationSession>> {
        let result = Entity::find()
            .filter(Column::ImpersonationSessionId.eq(session_id))
            .filter(Column::EndedAt.is_null())
            .one(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn find_active_by_admin(&self, admin_id: Uuid) -> Result<Vec<ImpersonationSession>> {
        let results = Entity::find()
            .filter(Column::AdminId.eq(admin_id))
            .filter(Column::EndedAt.is_null())
            .all(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn find_by_target_user(&self, target_user_id: Uuid) -> Result<Vec<ImpersonationSession>> {
        let results = Entity::find()
            .filter(Column::TargetUserId.eq(target_user_id))
            .all(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn end_session(
        &self,
        id: Uuid,
        ended_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<ImpersonationSession> {
        use sea_orm::ActiveValue::NotSet;

        let active_model = ActiveModel {
            id: Set(id),
            ended_at: Set(Some(ended_at)),
            admin_id: NotSet,
            target_user_id: NotSet,
            original_session_id: NotSet,
            impersonation_session_id: NotSet,
            reason: NotSet,
            started_at: NotSet,
        };

        let result = active_model
            .update(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.into())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        Entity::delete_by_id(id)
            .exec(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(())
    }
}
