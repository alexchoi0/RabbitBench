use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;
use tsa_core::{Result, Session, SessionRepository, TsaError};
use uuid::Uuid;

use crate::entity::session::{ActiveModel, Column, Entity};

#[derive(Clone)]
pub struct SeaOrmSessionRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmSessionRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

impl From<crate::entity::session::Model> for Session {
    fn from(model: crate::entity::session::Model) -> Self {
        Session {
            id: model.id,
            user_id: model.user_id,
            token_hash: model.token_hash,
            expires_at: model.expires_at,
            created_at: model.created_at,
            ip_address: model.ip_address,
            user_agent: model.user_agent,
        }
    }
}

#[async_trait]
impl SessionRepository for SeaOrmSessionRepository {
    async fn create(&self, session: &Session) -> Result<Session> {
        let active_model = ActiveModel {
            id: Set(session.id),
            user_id: Set(session.user_id),
            token_hash: Set(session.token_hash.clone()),
            expires_at: Set(session.expires_at),
            created_at: Set(session.created_at),
            ip_address: Set(session.ip_address.clone()),
            user_agent: Set(session.user_agent.clone()),
        };

        let result = active_model
            .insert(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.into())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Session>> {
        let result = Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<Session>> {
        let result = Entity::find()
            .filter(Column::TokenHash.eq(token_hash))
            .one(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Session>> {
        let results = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .all(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn update(&self, session: &Session) -> Result<Session> {
        let active_model = ActiveModel {
            id: Set(session.id),
            user_id: Set(session.user_id),
            token_hash: Set(session.token_hash.clone()),
            expires_at: Set(session.expires_at),
            created_at: Set(session.created_at),
            ip_address: Set(session.ip_address.clone()),
            user_agent: Set(session.user_agent.clone()),
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

    async fn delete_by_user_id(&self, user_id: Uuid) -> Result<()> {
        Entity::delete_many()
            .filter(Column::UserId.eq(user_id))
            .exec(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(())
    }

    async fn delete_expired(&self) -> Result<u64> {
        let result = Entity::delete_many()
            .filter(Column::ExpiresAt.lt(Utc::now()))
            .exec(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.rows_affected)
    }
}
