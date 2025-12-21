use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;
use tsa_core::{Result, TsaError, TwoFactor, TwoFactorRepository};
use uuid::Uuid;

use crate::entity::two_factor::{ActiveModel, Column, Entity};

#[derive(Clone)]
pub struct SeaOrmTwoFactorRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmTwoFactorRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

impl From<crate::entity::two_factor::Model> for TwoFactor {
    fn from(model: crate::entity::two_factor::Model) -> Self {
        TwoFactor {
            id: model.id,
            user_id: model.user_id,
            secret: model.secret,
            backup_codes: model.backup_codes,
            enabled: model.enabled,
            verified: model.verified,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[async_trait]
impl TwoFactorRepository for SeaOrmTwoFactorRepository {
    async fn create(&self, two_factor: &TwoFactor) -> Result<TwoFactor> {
        let active_model = ActiveModel {
            id: Set(two_factor.id),
            user_id: Set(two_factor.user_id),
            secret: Set(two_factor.secret.clone()),
            backup_codes: Set(two_factor.backup_codes.clone()),
            enabled: Set(two_factor.enabled),
            verified: Set(two_factor.verified),
            created_at: Set(two_factor.created_at),
            updated_at: Set(two_factor.updated_at),
        };

        let result = active_model
            .insert(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.into())
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<TwoFactor>> {
        let result = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .one(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn update(&self, two_factor: &TwoFactor) -> Result<TwoFactor> {
        let active_model = ActiveModel {
            id: Set(two_factor.id),
            user_id: Set(two_factor.user_id),
            secret: Set(two_factor.secret.clone()),
            backup_codes: Set(two_factor.backup_codes.clone()),
            enabled: Set(two_factor.enabled),
            verified: Set(two_factor.verified),
            created_at: Set(two_factor.created_at),
            updated_at: Set(two_factor.updated_at),
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
}
