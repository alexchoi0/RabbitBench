use async_trait::async_trait;
use tsa_core::{Result, User};

#[async_trait]
pub trait AuthCallbacks: Send + Sync {
    async fn on_user_created(&self, _user: &User) -> Result<()> {
        Ok(())
    }

    async fn send_verification_email(&self, user: &User, token: &str) -> Result<()>;

    async fn send_password_reset_email(&self, user: &User, token: &str) -> Result<()>;

    async fn send_magic_link_email(&self, _user: &User, _token: &str) -> Result<()> {
        Ok(())
    }

    async fn send_otp_email(&self, _user: &User, _code: &str) -> Result<()> {
        Ok(())
    }

    async fn send_phone_otp(&self, _phone: &str, _code: &str) -> Result<()> {
        Ok(())
    }
}

pub struct NoopCallbacks;

#[async_trait]
impl AuthCallbacks for NoopCallbacks {
    async fn send_verification_email(&self, _user: &User, _token: &str) -> Result<()> {
        Ok(())
    }

    async fn send_password_reset_email(&self, _user: &User, _token: &str) -> Result<()> {
        Ok(())
    }
}
