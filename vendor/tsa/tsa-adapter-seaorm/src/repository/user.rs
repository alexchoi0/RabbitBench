use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;
use tsa_core::{Result, TsaError, User, UserRepository};
use uuid::Uuid;

use crate::entity::user::{ActiveModel, Column, Entity};

#[derive(Clone)]
pub struct SeaOrmUserRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmUserRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

impl From<crate::entity::user::Model> for User {
    fn from(model: crate::entity::user::Model) -> Self {
        User {
            id: model.id,
            email: model.email,
            email_verified: model.email_verified,
            phone: model.phone,
            phone_verified: model.phone_verified,
            name: model.name,
            image: model.image,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[async_trait]
impl UserRepository for SeaOrmUserRepository {
    async fn create(&self, user: &User) -> Result<User> {
        let active_model = ActiveModel {
            id: Set(user.id),
            email: Set(user.email.clone()),
            email_verified: Set(user.email_verified),
            phone: Set(user.phone.clone()),
            phone_verified: Set(user.phone_verified),
            name: Set(user.name.clone()),
            image: Set(user.image.clone()),
            created_at: Set(user.created_at),
            updated_at: Set(user.updated_at),
        };

        let result = active_model.insert(self.db.as_ref()).await.map_err(|e| {
            if e.to_string().contains("duplicate") || e.to_string().contains("UNIQUE") {
                TsaError::UserAlreadyExists
            } else {
                TsaError::Database(e.to_string())
            }
        })?;

        Ok(result.into())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let result = Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let result = Entity::find()
            .filter(Column::Email.eq(email))
            .one(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn find_by_phone(&self, phone: &str) -> Result<Option<User>> {
        let result = Entity::find()
            .filter(Column::Phone.eq(phone))
            .one(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn update(&self, user: &User) -> Result<User> {
        let active_model = ActiveModel {
            id: Set(user.id),
            email: Set(user.email.clone()),
            email_verified: Set(user.email_verified),
            phone: Set(user.phone.clone()),
            phone_verified: Set(user.phone_verified),
            name: Set(user.name.clone()),
            image: Set(user.image.clone()),
            created_at: Set(user.created_at),
            updated_at: Set(user.updated_at),
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
