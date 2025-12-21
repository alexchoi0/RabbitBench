use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub email: String,
    pub email_verified: bool,
    #[sea_orm(unique)]
    pub phone: Option<String>,
    pub phone_verified: bool,
    pub name: Option<String>,
    pub image: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::session::Entity")]
    Sessions,
    #[sea_orm(has_many = "super::account::Entity")]
    Accounts,
    #[sea_orm(has_many = "super::verification_token::Entity")]
    VerificationTokens,
    #[sea_orm(has_one = "super::two_factor::Entity")]
    TwoFactor,
    #[sea_orm(has_many = "super::organization_member::Entity")]
    OrganizationMembers,
    #[sea_orm(has_many = "super::api_key::Entity")]
    ApiKeys,
    #[sea_orm(has_many = "super::passkey::Entity")]
    Passkeys,
    #[sea_orm(has_many = "super::passkey_challenge::Entity")]
    PasskeyChallenges,
}

impl Related<super::session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sessions.def()
    }
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Accounts.def()
    }
}

impl Related<super::verification_token::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::VerificationTokens.def()
    }
}

impl Related<super::two_factor::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TwoFactor.def()
    }
}

impl Related<super::organization_member::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OrganizationMembers.def()
    }
}

impl Related<super::api_key::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ApiKeys.def()
    }
}

impl Related<super::passkey::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Passkeys.def()
    }
}

impl Related<super::passkey_challenge::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PasskeyChallenges.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
