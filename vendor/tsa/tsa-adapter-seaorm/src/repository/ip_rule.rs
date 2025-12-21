use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;
use tsa_core::{IpRule, IpRuleRepository, IpRuleType, Result, TsaError};
use uuid::Uuid;

use crate::entity::ip_rule::{ActiveModel, Column, Entity};

#[derive(Clone)]
pub struct SeaOrmIpRuleRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmIpRuleRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

impl From<crate::entity::ip_rule::Model> for IpRule {
    fn from(model: crate::entity::ip_rule::Model) -> Self {
        let rule_type = match model.rule_type.as_str() {
            "allow" => IpRuleType::Allow,
            "block" => IpRuleType::Block,
            _ => IpRuleType::Block,
        };
        IpRule {
            id: model.id,
            ip_pattern: model.ip_pattern,
            rule_type,
            description: model.description,
            expires_at: model.expires_at,
            created_by: model.created_by,
            created_at: model.created_at,
        }
    }
}

#[async_trait]
impl IpRuleRepository for SeaOrmIpRuleRepository {
    async fn create(&self, rule: &IpRule) -> Result<IpRule> {
        let rule_type_str = match rule.rule_type {
            IpRuleType::Allow => "allow",
            IpRuleType::Block => "block",
        };

        let active_model = ActiveModel {
            id: Set(rule.id),
            ip_pattern: Set(rule.ip_pattern.clone()),
            rule_type: Set(rule_type_str.to_string()),
            description: Set(rule.description.clone()),
            expires_at: Set(rule.expires_at),
            created_by: Set(rule.created_by),
            created_at: Set(rule.created_at),
        };

        let result = active_model
            .insert(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.into())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<IpRule>> {
        let result = Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    async fn find_all(&self) -> Result<Vec<IpRule>> {
        let results = Entity::find()
            .all(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn find_by_type(&self, rule_type: IpRuleType) -> Result<Vec<IpRule>> {
        let rule_type_str = match rule_type {
            IpRuleType::Allow => "allow",
            IpRuleType::Block => "block",
        };

        let results = Entity::find()
            .filter(Column::RuleType.eq(rule_type_str))
            .all(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn find_active(&self) -> Result<Vec<IpRule>> {
        let now = Utc::now();
        let results = Entity::find()
            .filter(Column::ExpiresAt.is_null().or(Column::ExpiresAt.gt(now)))
            .all(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn update(&self, rule: &IpRule) -> Result<IpRule> {
        let rule_type_str = match rule.rule_type {
            IpRuleType::Allow => "allow",
            IpRuleType::Block => "block",
        };

        let active_model = ActiveModel {
            id: Set(rule.id),
            ip_pattern: Set(rule.ip_pattern.clone()),
            rule_type: Set(rule_type_str.to_string()),
            description: Set(rule.description.clone()),
            expires_at: Set(rule.expires_at),
            created_by: Set(rule.created_by),
            created_at: Set(rule.created_at),
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

    async fn delete_expired(&self) -> Result<u64> {
        let now = Utc::now();
        let result = Entity::delete_many()
            .filter(Column::ExpiresAt.lt(now))
            .exec(self.db.as_ref())
            .await
            .map_err(|e| TsaError::Database(e.to_string()))?;

        Ok(result.rows_affected)
    }
}
