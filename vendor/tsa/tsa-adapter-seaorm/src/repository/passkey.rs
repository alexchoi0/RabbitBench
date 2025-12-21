use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;
use tsa_core::{Passkey, PasskeyRepository, Result, TsaError};
use uuid::Uuid;

use crate::entity::passkey::{ActiveModel, Column, Entity};

#[derive(Clone)]
pub struct SeaOrmPasskeyRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmPasskeyRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

impl From<crate::entity::passkey::Model> for Passkey {
    fn from(model: crate::entity::passkey::Model) -> Self {
        Passkey {
            id: model.id,
            user_id: model.user_id,
            credential_id: model.credential_id,
            public_key: model.public_key,
            counter: model.counter as u32,
            name: model.name,
            transports: model
                .transports
                .map(|t| serde_json::from_str(&t).unwrap_or_default()),
            created_at: model.created_at,
            last_used_at: model.last_used_at,
        }
    }
}

#[async_trait]
impl PasskeyRepository for SeaOrmPasskeyRepository {
    async fn create(&self, passkey: &Passkey) -> Result<Passkey> {
        let transports_json = passkey
            .transports
            .as_ref()
            .map(|t| serde_json::to_string(t).unwrap_or_default());

        let active_model = ActiveModel {
            id: Set(passkey.id),
            user_id: Set(passkey.user_id),
            credential_id: Set(passkey.credential_id.clone()),
            public_key: Set(passkey.public_key.clone()),
            counter: Set(passkey.counter as i64),
            name: Set(passkey.name.clone()),
            transports: Set(transports_json),
            created_at: Set(passkey.created_at),
            last_used_at: Set(passkey.last_used_at),
        };

        let result = active_model.insert(self.db.as_ref()).await.map_err(|e| {
            if e.to_string().contains("duplicate") || e.to_string().contains("UNIQUE") {
                TsaError::PasskeyAlreadyRegistered
            } else {
                TsaError::Database(e.to_string())
            }
        })?;

        Ok(result.into())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Passkey>> {
        let result = Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn find_by_credential_id(&self, credential_id: &[u8]) -> Result<Option<Passkey>> {
        let result = Entity::find()
            .filter(Column::CredentialId.eq(credential_id.to_vec()))
            .one(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Passkey>> {
        let results = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .all(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn update(&self, passkey: &Passkey) -> Result<Passkey> {
        let transports_json = passkey
            .transports
            .as_ref()
            .map(|t| serde_json::to_string(t).unwrap_or_default());

        let active_model = ActiveModel {
            id: Set(passkey.id),
            user_id: Set(passkey.user_id),
            credential_id: Set(passkey.credential_id.clone()),
            public_key: Set(passkey.public_key.clone()),
            counter: Set(passkey.counter as i64),
            name: Set(passkey.name.clone()),
            transports: Set(transports_json),
            created_at: Set(passkey.created_at),
            last_used_at: Set(passkey.last_used_at),
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

    async fn delete_by_user(&self, user_id: Uuid) -> Result<()> {
        Entity::delete_many()
            .filter(Column::UserId.eq(user_id))
            .exec(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(())
    }
}
