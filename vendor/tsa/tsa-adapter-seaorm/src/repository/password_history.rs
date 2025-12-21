use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use std::sync::Arc;
use tsa_core::{PasswordHistory, PasswordHistoryRepository, Result, TsaError};
use uuid::Uuid;

use crate::entity::password_history::{ActiveModel, Column, Entity};

#[derive(Clone)]
pub struct SeaOrmPasswordHistoryRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmPasswordHistoryRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

impl From<crate::entity::password_history::Model> for PasswordHistory {
    fn from(model: crate::entity::password_history::Model) -> Self {
        PasswordHistory {
            id: model.id,
            user_id: model.user_id,
            password_hash: model.password_hash,
            created_at: model.created_at,
        }
    }
}

#[async_trait]
impl PasswordHistoryRepository for SeaOrmPasswordHistoryRepository {
    async fn create(&self, history: &PasswordHistory) -> Result<PasswordHistory> {
        let active_model = ActiveModel {
            id: Set(history.id),
            user_id: Set(history.user_id),
            password_hash: Set(history.password_hash.clone()),
            created_at: Set(history.created_at),
        };

        let result = active_model
            .insert(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.into())
    }

    async fn find_by_user(&self, user_id: Uuid, limit: u32) -> Result<Vec<PasswordHistory>> {
        let results = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .order_by_desc(Column::CreatedAt)
            .limit(limit as u64)
            .all(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn delete_old_entries(&self, user_id: Uuid, keep_count: u32) -> Result<u64> {
        let all_entries: Vec<crate::entity::password_history::Model> = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .order_by_desc(Column::CreatedAt)
            .all(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        if all_entries.len() <= keep_count as usize {
            return Ok(0);
        }

        let to_delete: Vec<Uuid> = all_entries
            .into_iter()
            .skip(keep_count as usize)
            .map(|e| e.id)
            .collect();

        let count = to_delete.len() as u64;

        for id in to_delete {
            Entity::delete_by_id(id)
                .exec(self.db.as_ref())
                .await
                .map_err(|e| TsaError::Database(e.to_string()))?;
        }

        Ok(count)
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
