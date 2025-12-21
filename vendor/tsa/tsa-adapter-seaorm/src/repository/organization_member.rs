use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;
use tsa_core::{
    OrganizationMember, OrganizationMemberRepository, OrganizationRole, Result, TsaError,
};
use uuid::Uuid;

use crate::entity::organization_member::{ActiveModel, Column, Entity};

#[derive(Clone)]
pub struct SeaOrmOrganizationMemberRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmOrganizationMemberRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

impl From<crate::entity::organization_member::Model> for OrganizationMember {
    fn from(model: crate::entity::organization_member::Model) -> Self {
        OrganizationMember {
            id: model.id,
            organization_id: model.organization_id,
            user_id: model.user_id,
            role: model.role.parse().unwrap_or(OrganizationRole::Member),
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[async_trait]
impl OrganizationMemberRepository for SeaOrmOrganizationMemberRepository {
    async fn create(&self, member: &OrganizationMember) -> Result<OrganizationMember> {
        let active_model = ActiveModel {
            id: Set(member.id),
            organization_id: Set(member.organization_id),
            user_id: Set(member.user_id),
            role: Set(member.role.to_string()),
            created_at: Set(member.created_at),
            updated_at: Set(member.updated_at),
        };

        let result = active_model.insert(self.db.as_ref()).await.map_err(|e| {
            if e.to_string().contains("duplicate") || e.to_string().contains("UNIQUE") {
                TsaError::AlreadyOrganizationMember
            } else {
                TsaError::Database(e.to_string())
            }
        })?;

        Ok(result.into())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<OrganizationMember>> {
        let result = Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn find_by_org_and_user(
        &self,
        organization_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<OrganizationMember>> {
        let result = Entity::find()
            .filter(Column::OrganizationId.eq(organization_id))
            .filter(Column::UserId.eq(user_id))
            .one(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn find_by_organization(&self, organization_id: Uuid) -> Result<Vec<OrganizationMember>> {
        let results = Entity::find()
            .filter(Column::OrganizationId.eq(organization_id))
            .all(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<OrganizationMember>> {
        let results = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .all(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn update(&self, member: &OrganizationMember) -> Result<OrganizationMember> {
        let active_model = ActiveModel {
            id: Set(member.id),
            organization_id: Set(member.organization_id),
            user_id: Set(member.user_id),
            role: Set(member.role.to_string()),
            created_at: Set(member.created_at),
            updated_at: Set(member.updated_at),
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

    async fn delete_by_organization(&self, organization_id: Uuid) -> Result<()> {
        Entity::delete_many()
            .filter(Column::OrganizationId.eq(organization_id))
            .exec(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(())
    }
}
