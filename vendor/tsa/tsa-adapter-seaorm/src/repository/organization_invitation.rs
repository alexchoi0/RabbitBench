use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;
use tsa_core::{
    InvitationStatus, OrganizationInvitation, OrganizationInvitationRepository, OrganizationRole,
    Result, TsaError,
};
use uuid::Uuid;

use crate::entity::organization_invitation::{ActiveModel, Column, Entity};

#[derive(Clone)]
pub struct SeaOrmOrganizationInvitationRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmOrganizationInvitationRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

impl From<crate::entity::organization_invitation::Model> for OrganizationInvitation {
    fn from(model: crate::entity::organization_invitation::Model) -> Self {
        OrganizationInvitation {
            id: model.id,
            organization_id: model.organization_id,
            email: model.email,
            role: model.role.parse().unwrap_or(OrganizationRole::Member),
            token_hash: model.token_hash,
            invited_by: model.invited_by,
            status: model.status.parse().unwrap_or(InvitationStatus::Pending),
            expires_at: model.expires_at,
            created_at: model.created_at,
        }
    }
}

#[async_trait]
impl OrganizationInvitationRepository for SeaOrmOrganizationInvitationRepository {
    async fn create(&self, invitation: &OrganizationInvitation) -> Result<OrganizationInvitation> {
        let active_model = ActiveModel {
            id: Set(invitation.id),
            organization_id: Set(invitation.organization_id),
            email: Set(invitation.email.clone()),
            role: Set(invitation.role.to_string()),
            token_hash: Set(invitation.token_hash.clone()),
            invited_by: Set(invitation.invited_by),
            status: Set(invitation.status.to_string()),
            expires_at: Set(invitation.expires_at),
            created_at: Set(invitation.created_at),
        };

        let result = active_model
            .insert(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.into())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<OrganizationInvitation>> {
        let result = Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<OrganizationInvitation>> {
        let result = Entity::find()
            .filter(Column::TokenHash.eq(token_hash))
            .one(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn find_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<OrganizationInvitation>> {
        let results = Entity::find()
            .filter(Column::OrganizationId.eq(organization_id))
            .all(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn find_by_email(&self, email: &str) -> Result<Vec<OrganizationInvitation>> {
        let results = Entity::find()
            .filter(Column::Email.eq(email))
            .all(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn find_pending_by_org_and_email(
        &self,
        organization_id: Uuid,
        email: &str,
    ) -> Result<Option<OrganizationInvitation>> {
        let result = Entity::find()
            .filter(Column::OrganizationId.eq(organization_id))
            .filter(Column::Email.eq(email))
            .filter(Column::Status.eq("pending"))
            .one(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn update(&self, invitation: &OrganizationInvitation) -> Result<OrganizationInvitation> {
        let active_model = ActiveModel {
            id: Set(invitation.id),
            organization_id: Set(invitation.organization_id),
            email: Set(invitation.email.clone()),
            role: Set(invitation.role.to_string()),
            token_hash: Set(invitation.token_hash.clone()),
            invited_by: Set(invitation.invited_by),
            status: Set(invitation.status.to_string()),
            expires_at: Set(invitation.expires_at),
            created_at: Set(invitation.created_at),
        };

        let result = active_model
            .update(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.into())
    }

    async fn update_status(&self, id: Uuid, status: InvitationStatus) -> Result<()> {
        use sea_orm::ActiveValue::NotSet;

        let active_model = ActiveModel {
            id: Set(id),
            status: Set(status.to_string()),
            organization_id: NotSet,
            email: NotSet,
            role: NotSet,
            token_hash: NotSet,
            invited_by: NotSet,
            expires_at: NotSet,
            created_at: NotSet,
        };

        active_model
            .update(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        Entity::delete_by_id(id)
            .exec(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(())
    }

    async fn delete_expired(&self) -> Result<u64> {
        let now = Utc::now();
        let result = Entity::delete_many()
            .filter(Column::ExpiresAt.lt(now))
            .filter(Column::Status.eq("pending"))
            .exec(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.rows_affected)
    }
}
