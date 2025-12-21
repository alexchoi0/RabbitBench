use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;
use tsa_core::{AccountLockout, AccountLockoutRepository, Result, TsaError};
use uuid::Uuid;

use crate::entity::account_lockout::{ActiveModel, Column, Entity};

#[derive(Clone)]
pub struct SeaOrmAccountLockoutRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmAccountLockoutRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

impl From<crate::entity::account_lockout::Model> for AccountLockout {
    fn from(model: crate::entity::account_lockout::Model) -> Self {
        AccountLockout {
            id: model.id,
            user_id: model.user_id,
            failed_attempts: model.failed_attempts as u32,
            locked_until: model.locked_until,
            last_failed_at: Some(model.updated_at),
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[async_trait]
impl AccountLockoutRepository for SeaOrmAccountLockoutRepository {
    async fn create(&self, lockout: &AccountLockout) -> Result<AccountLockout> {
        let active_model = ActiveModel {
            id: Set(lockout.id),
            user_id: Set(lockout.user_id),
            failed_attempts: Set(lockout.failed_attempts as i32),
            locked_until: Set(lockout.locked_until),
            created_at: Set(lockout.created_at),
            updated_at: Set(lockout.updated_at),
        };

        let result = active_model
            .insert(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.into())
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Option<AccountLockout>> {
        let result = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .one(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn update(&self, lockout: &AccountLockout) -> Result<AccountLockout> {
        let active_model = ActiveModel {
            id: Set(lockout.id),
            user_id: Set(lockout.user_id),
            failed_attempts: Set(lockout.failed_attempts as i32),
            locked_until: Set(lockout.locked_until),
            created_at: Set(lockout.created_at),
            updated_at: Set(lockout.updated_at),
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

    async fn increment_failed_attempts(&self, user_id: Uuid) -> Result<AccountLockout> {
        let existing = self.find_by_user(user_id).await?;

        match existing {
            Some(lockout) => {
                let updated = AccountLockout {
                    failed_attempts: lockout.failed_attempts + 1,
                    updated_at: Utc::now(),
                    ..lockout
                };
                self.update(&updated).await
            }
            None => {
                let now = Utc::now();
                let lockout = AccountLockout {
                    id: Uuid::new_v4(),
                    user_id,
                    failed_attempts: 1,
                    locked_until: None,
                    last_failed_at: Some(now),
                    created_at: now,
                    updated_at: now,
                };
                self.create(&lockout).await
            }
        }
    }

    async fn reset_failed_attempts(&self, user_id: Uuid) -> Result<()> {
        self.delete_by_user(user_id).await
    }
}
