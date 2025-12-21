use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "organizations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    #[sea_orm(unique)]
    pub slug: String,
    pub logo: Option<String>,
    pub metadata: Option<Json>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::organization_member::Entity")]
    Members,
    #[sea_orm(has_many = "super::organization_invitation::Entity")]
    Invitations,
    #[sea_orm(has_many = "super::api_key::Entity")]
    ApiKeys,
}

impl Related<super::organization_member::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Members.def()
    }
}

impl Related<super::organization_invitation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Invitations.def()
    }
}

impl Related<super::api_key::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ApiKeys.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
