use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
}

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct OptionalAuthUser(pub Option<AuthUser>);

pub struct AuthError(pub String);

impl From<AuthError> for (StatusCode, String) {
    fn from(err: AuthError) -> Self {
        (StatusCode::UNAUTHORIZED, err.0)
    }
}

pub fn validate_token(token: &str, secret: &str) -> Result<AuthUser, AuthError> {
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());
    let mut validation = Validation::default();
    validation.validate_exp = true;

    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|e| AuthError(format!("Invalid token: {}", e)))?;

    Ok(AuthUser {
        user_id: token_data.claims.sub,
    })
}

impl<S> FromRequestParts<S> for OptionalAuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok());

        let Some(auth_header) = auth_header else {
            return Ok(OptionalAuthUser(None));
        };

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or((StatusCode::UNAUTHORIZED, "Invalid authorization header".to_string()))?;

        let secret = parts
            .extensions
            .get::<String>()
            .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Auth secret not configured".to_string()))?;

        match validate_token(token, secret) {
            Ok(user) => Ok(OptionalAuthUser(Some(user))),
            Err(_) => Ok(OptionalAuthUser(None)),
        }
    }
}
