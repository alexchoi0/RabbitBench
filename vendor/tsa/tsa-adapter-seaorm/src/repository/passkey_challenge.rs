use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;
use tsa_core::{
    PasskeyChallenge, PasskeyChallengeRepository, PasskeyChallengeType, Result, TsaError,
};
use uuid::Uuid;

use crate::entity::passkey_challenge::{ActiveModel, Column, Entity};

#[derive(Clone)]
pub struct SeaOrmPasskeyChallengeRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmPasskeyChallengeRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

impl From<crate::entity::passkey_challenge::Model> for PasskeyChallenge {
    fn from(model: crate::entity::passkey_challenge::Model) -> Self {
        let challenge_type = match model.challenge_type.as_str() {
            "registration" => PasskeyChallengeType::Registration,
            "authentication" => PasskeyChallengeType::Authentication,
            _ => PasskeyChallengeType::Authentication,
        };

        PasskeyChallenge {
            id: model.id,
            user_id: model.user_id,
            challenge: model.challenge,
            challenge_type,
            state: model.state,
            expires_at: model.expires_at,
            created_at: model.created_at,
        }
    }
}

fn challenge_type_to_string(ct: &PasskeyChallengeType) -> String {
    match ct {
        PasskeyChallengeType::Registration => "registration".to_string(),
        PasskeyChallengeType::Authentication => "authentication".to_string(),
    }
}

#[async_trait]
impl PasskeyChallengeRepository for SeaOrmPasskeyChallengeRepository {
    async fn create(&self, challenge: &PasskeyChallenge) -> Result<PasskeyChallenge> {
        let active_model = ActiveModel {
            id: Set(challenge.id),
            user_id: Set(challenge.user_id),
            challenge: Set(challenge.challenge.clone()),
            challenge_type: Set(challenge_type_to_string(&challenge.challenge_type)),
            state: Set(challenge.state.clone()),
            expires_at: Set(challenge.expires_at),
            created_at: Set(challenge.created_at),
        };

        let result = active_model
            .insert(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.into())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<PasskeyChallenge>> {
        let result = Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn find_by_challenge(&self, challenge: &[u8]) -> Result<Option<PasskeyChallenge>> {
        let result = Entity::find()
            .filter(Column::Challenge.eq(challenge.to_vec()))
            .one(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        Entity::delete_by_id(id)
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
