use chrono::Duration;

#[derive(Clone)]
pub struct AuthConfig {
    pub app_name: String,
    pub session_expiry: Duration,
    pub session_token_length: usize,
    pub verification_token_expiry: Duration,
    pub password_reset_token_expiry: Duration,
    pub magic_link_expiry: Duration,
    pub otp_expiry: Duration,
    pub require_email_verification: bool,
    pub enable_session_refresh: bool,
    pub session_refresh_threshold: Duration,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            app_name: "TSA".to_string(),
            session_expiry: Duration::days(30),
            session_token_length: 32,
            verification_token_expiry: Duration::hours(24),
            password_reset_token_expiry: Duration::hours(1),
            magic_link_expiry: Duration::minutes(15),
            otp_expiry: Duration::minutes(10),
            require_email_verification: false,
            enable_session_refresh: true,
            session_refresh_threshold: Duration::days(7),
        }
    }
}

impl AuthConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn app_name(mut self, name: impl Into<String>) -> Self {
        self.app_name = name.into();
        self
    }

    pub fn session_expiry(mut self, duration: Duration) -> Self {
        self.session_expiry = duration;
        self
    }

    pub fn require_email_verification(mut self, require: bool) -> Self {
        self.require_email_verification = require;
        self
    }

    pub fn verification_token_expiry(mut self, duration: Duration) -> Self {
        self.verification_token_expiry = duration;
        self
    }

    pub fn password_reset_token_expiry(mut self, duration: Duration) -> Self {
        self.password_reset_token_expiry = duration;
        self
    }

    pub fn enable_session_refresh(mut self, enable: bool) -> Self {
        self.enable_session_refresh = enable;
        self
    }

    pub fn session_refresh_threshold(mut self, duration: Duration) -> Self {
        self.session_refresh_threshold = duration;
        self
    }

    pub fn magic_link_expiry(mut self, duration: Duration) -> Self {
        self.magic_link_expiry = duration;
        self
    }

    pub fn otp_expiry(mut self, duration: Duration) -> Self {
        self.otp_expiry = duration;
        self
    }
}
