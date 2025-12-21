use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;
use tsa_core::{Account, AccountRepository, Result, TsaError};
use uuid::Uuid;

use crate::entity::account::{ActiveModel, Column, Entity};

#[derive(Clone)]
pub struct SeaOrmAccountRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmAccountRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

impl From<crate::entity::account::Model> for Account {
    fn from(model: crate::entity::account::Model) -> Self {
        Account {
            id: model.id,
            user_id: model.user_id,
            provider: model.provider,
            provider_account_id: model.provider_account_id,
            access_token: model.access_token,
            refresh_token: model.refresh_token,
            expires_at: model.expires_at,
            created_at: model.created_at,
        }
    }
}

#[async_trait]
impl AccountRepository for SeaOrmAccountRepository {
    async fn create(&self, account: &Account) -> Result<Account> {
        let now = chrono::Utc::now();
        let active_model = ActiveModel {
            id: Set(account.id),
            user_id: Set(account.user_id),
            provider: Set(account.provider.clone()),
            provider_account_id: Set(account.provider_account_id.clone()),
            password_hash: Set(None),
            access_token: Set(account.access_token.clone()),
            refresh_token: Set(account.refresh_token.clone()),
            expires_at: Set(account.expires_at),
            token_type: Set(None),
            scope: Set(None),
            id_token: Set(None),
            created_at: Set(account.created_at),
            updated_at: Set(now),
        };

        let result = active_model
            .insert(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.into())
    }

    async fn find_by_provider(
        &self,
        provider: &str,
        provider_account_id: &str,
    ) -> Result<Option<Account>> {
        let result = Entity::find()
            .filter(Column::Provider.eq(provider))
            .filter(Column::ProviderAccountId.eq(provider_account_id))
            .one(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Account>> {
        let results = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .all(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        Entity::delete_by_id(id)
            .exec(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(())
    }
}
