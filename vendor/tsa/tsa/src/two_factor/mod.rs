mod backup_codes;
#[cfg(feature = "totp")]
mod totp;

pub use backup_codes::*;
#[cfg(feature = "totp")]
pub use totp::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoFactorSetup {
    pub secret: String,
    pub otpauth_url: String,
    pub backup_codes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TwoFactorMethod {
    Totp,
    BackupCode,
}
