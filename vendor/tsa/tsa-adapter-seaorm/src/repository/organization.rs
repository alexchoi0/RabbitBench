use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;
use tsa_core::{Organization, OrganizationRepository, Result, TsaError};
use uuid::Uuid;

use crate::entity::organization::{ActiveModel, Column, Entity};

#[derive(Clone)]
pub struct SeaOrmOrganizationRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmOrganizationRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

impl From<crate::entity::organization::Model> for Organization {
    fn from(model: crate::entity::organization::Model) -> Self {
        Organization {
            id: model.id,
            name: model.name,
            slug: model.slug,
            logo: model.logo,
            metadata: model.metadata,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[async_trait]
impl OrganizationRepository for SeaOrmOrganizationRepository {
    async fn create(&self, organization: &Organization) -> Result<Organization> {
        let active_model = ActiveModel {
            id: Set(organization.id),
            name: Set(organization.name.clone()),
            slug: Set(organization.slug.clone()),
            logo: Set(organization.logo.clone()),
            metadata: Set(organization.metadata.clone()),
            created_at: Set(organization.created_at),
            updated_at: Set(organization.updated_at),
        };

        let result = active_model.insert(self.db.as_ref()).await.map_err(|e| {
            if e.to_string().contains("duplicate") || e.to_string().contains("UNIQUE") {
                TsaError::OrganizationAlreadyExists
            } else {
                TsaError::Database(e.to_string())
            }
        })?;

        Ok(result.into())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Organization>> {
        let result = Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<Organization>> {
        let result = Entity::find()
            .filter(Column::Slug.eq(slug))
            .one(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn update(&self, organization: &Organization) -> Result<Organization> {
        let active_model = ActiveModel {
            id: Set(organization.id),
            name: Set(organization.name.clone()),
            slug: Set(organization.slug.clone()),
            logo: Set(organization.logo.clone()),
            metadata: Set(organization.metadata.clone()),
            created_at: Set(organization.created_at),
            updated_at: Set(organization.updated_at),
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
