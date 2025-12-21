use base64::{engine::general_purpose::STANDARD, Engine};
use totp_rs::{Algorithm, Secret, TOTP};
use tsa_core::{Result, TsaError};

pub struct TotpManager {
    issuer: String,
    digits: usize,
    step: u64,
}

impl TotpManager {
    pub fn new(issuer: impl Into<String>) -> Self {
        Self {
            issuer: issuer.into(),
            digits: 6,
            step: 30,
        }
    }

    pub fn with_digits(mut self, digits: usize) -> Self {
        self.digits = digits;
        self
    }

    pub fn with_step(mut self, step: u64) -> Self {
        self.step = step;
        self
    }

    pub fn generate_secret(&self) -> String {
        let secret = Secret::generate_secret();
        STANDARD.encode(secret.to_bytes().unwrap())
    }

    pub fn get_otpauth_url(&self, secret: &str, account_name: &str) -> Result<String> {
        let totp = self.create_totp(secret, account_name)?;
        Ok(totp.get_url())
    }

    pub fn verify(&self, secret: &str, code: &str, account_name: &str) -> Result<bool> {
        let totp = self.create_totp(secret, account_name)?;

        let code = code.trim().replace(" ", "").replace("-", "");

        Ok(totp.check_current(&code).unwrap_or(false))
    }

    pub fn generate_current(&self, secret: &str, account_name: &str) -> Result<String> {
        let totp = self.create_totp(secret, account_name)?;
        totp.generate_current()
            .map_err(|e| TsaError::Internal(e.to_string()))
    }

    fn create_totp(&self, secret: &str, account_name: &str) -> Result<TOTP> {
        let secret_bytes = STANDARD
            .decode(secret)
            .map_err(|e| TsaError::Internal(format!("Invalid secret: {}", e)))?;

        TOTP::new(
            Algorithm::SHA1,
            self.digits,
            1,
            self.step,
            secret_bytes,
            Some(self.issuer.clone()),
            account_name.to_string(),
        )
        .map_err(|e| TsaError::Internal(e.to_string()))
    }
}

impl Default for TotpManager {
    fn default() -> Self {
        Self::new("TSA")
    }
}
