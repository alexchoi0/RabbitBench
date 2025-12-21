use async_graphql::{InputObject, SimpleObject, ID};

#[derive(SimpleObject)]
pub struct User {
    pub id: ID,
    pub email: String,
    pub name: Option<String>,
    pub email_verified: bool,
}

#[derive(SimpleObject)]
pub struct AuthPayload {
    pub user: User,
    pub session_token: String,
}

#[derive(SimpleObject)]
pub struct ApiKey {
    pub id: ID,
    pub name: String,
    pub prefix: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<String>,
    pub created_at: String,
}

#[derive(SimpleObject)]
pub struct CreateApiKeyPayload {
    pub api_key: ApiKey,
    pub secret: String,
}

#[derive(InputObject)]
pub struct SignupInput {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
}

#[derive(InputObject)]
pub struct SigninInput {
    pub email: String,
    pub password: String,
}

#[derive(InputObject)]
pub struct CreateApiKeyInput {
    pub name: String,
    #[graphql(default)]
    pub scopes: Vec<String>,
}

impl From<tsa_core::User> for User {
    fn from(u: tsa_core::User) -> Self {
        Self {
            id: u.id.to_string().into(),
            email: u.email,
            name: u.name,
            email_verified: u.email_verified,
        }
    }
}

impl From<tsa_core::ApiKey> for ApiKey {
    fn from(k: tsa_core::ApiKey) -> Self {
        Self {
            id: k.id.to_string().into(),
            name: k.name,
            prefix: k.prefix,
            scopes: k.scopes,
            expires_at: k.expires_at.map(|dt| dt.to_rfc3339()),
            created_at: k.created_at.to_rfc3339(),
        }
    }
}
