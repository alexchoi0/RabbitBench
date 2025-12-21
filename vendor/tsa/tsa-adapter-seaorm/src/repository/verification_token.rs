use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;
use tsa_core::{Result, TokenType, TsaError, VerificationToken, VerificationTokenRepository};
use uuid::Uuid;

use crate::entity::verification_token::{ActiveModel, Column, Entity};

#[derive(Clone)]
pub struct SeaOrmVerificationTokenRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmVerificationTokenRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

impl From<crate::entity::verification_token::TokenType> for TokenType {
    fn from(t: crate::entity::verification_token::TokenType) -> Self {
        match t {
            crate::entity::verification_token::TokenType::EmailVerification => {
                TokenType::EmailVerification
            }
            crate::entity::verification_token::TokenType::PasswordReset => TokenType::PasswordReset,
            crate::entity::verification_token::TokenType::MagicLink => TokenType::MagicLink,
            crate::entity::verification_token::TokenType::EmailOtp => TokenType::EmailOtp,
            crate::entity::verification_token::TokenType::PhoneOtp => TokenType::PhoneOtp,
        }
    }
}

impl From<TokenType> for crate::entity::verification_token::TokenType {
    fn from(t: TokenType) -> Self {
        match t {
            TokenType::EmailVerification => {
                crate::entity::verification_token::TokenType::EmailVerification
            }
            TokenType::PasswordReset => crate::entity::verification_token::TokenType::PasswordReset,
            TokenType::MagicLink => crate::entity::verification_token::TokenType::MagicLink,
            TokenType::EmailOtp => crate::entity::verification_token::TokenType::EmailOtp,
            TokenType::PhoneOtp => crate::entity::verification_token::TokenType::PhoneOtp,
        }
    }
}

impl From<crate::entity::verification_token::Model> for VerificationToken {
    fn from(model: crate::entity::verification_token::Model) -> Self {
        VerificationToken {
            id: model.id,
            user_id: model.user_id,
            token_hash: model.token_hash,
            token_type: model.token_type.into(),
            expires_at: model.expires_at,
            created_at: model.created_at,
        }
    }
}

#[async_trait]
impl VerificationTokenRepository for SeaOrmVerificationTokenRepository {
    async fn create(&self, token: &VerificationToken) -> Result<VerificationToken> {
        let active_model = ActiveModel {
            id: Set(token.id),
            user_id: Set(token.user_id),
            token_hash: Set(token.token_hash.clone()),
            token_type: Set(token.token_type.clone().into()),
            expires_at: Set(token.expires_at),
            created_at: Set(token.created_at),
        };

        let result = active_model
            .insert(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.into())
    }

    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<VerificationToken>> {
        let result = Entity::find()
            .filter(Column::TokenHash.eq(token_hash))
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
