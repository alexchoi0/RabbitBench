use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "impersonation_sessions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub admin_id: Uuid,
    pub target_user_id: Uuid,
    pub original_session_id: Uuid,
    #[sea_orm(unique)]
    pub impersonation_session_id: Uuid,
    pub reason: Option<String>,
    pub started_at: DateTimeUtc,
    pub ended_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::AdminId",
        to = "super::user::Column::Id"
    )]
    Admin,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::TargetUserId",
        to = "super::user::Column::Id"
    )]
    TargetUser,
    #[sea_orm(
        belongs_to = "super::session::Entity",
        from = "Column::OriginalSessionId",
        to = "super::session::Column::Id"
    )]
    OriginalSession,
    #[sea_orm(
        belongs_to = "super::session::Entity",
        from = "Column::ImpersonationSessionId",
        to = "super::session::Column::Id"
    )]
    ImpersonationSession,
}

impl ActiveModelBehavior for ActiveModel {}
