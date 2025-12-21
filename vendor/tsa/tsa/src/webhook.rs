use async_trait::async_trait;
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashSet;
use tokio::sync::mpsc;
use tsa_core::{Result, TsaError, WebhookEvent, WebhookPayload};

#[derive(Debug, Clone)]
pub struct WebhookConfig {
    pub url: String,
    pub secret: String,
    pub events: HashSet<WebhookEvent>,
    pub timeout_secs: u64,
    pub max_retries: u32,
}

impl WebhookConfig {
    pub fn new(url: impl Into<String>, secret: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            secret: secret.into(),
            events: HashSet::new(),
            timeout_secs: 30,
            max_retries: 3,
        }
    }

    pub fn with_events(mut self, events: impl IntoIterator<Item = WebhookEvent>) -> Self {
        self.events = events.into_iter().collect();
        self
    }

    pub fn with_all_events(mut self) -> Self {
        use WebhookEvent::*;
        self.events = [
            UserCreated,
            UserUpdated,
            UserDeleted,
            EmailVerified,
            SigninSuccess,
            SigninFailed,
            SignoutSuccess,
            PasswordChanged,
            PasswordResetRequested,
            SessionCreated,
            SessionRevoked,
            TwoFactorEnabled,
            TwoFactorDisabled,
            OrganizationCreated,
            OrganizationUpdated,
            OrganizationDeleted,
            MemberAdded,
            MemberRemoved,
            MemberRoleChanged,
            InvitationSent,
            InvitationAccepted,
            InvitationRevoked,
            ApiKeyCreated,
            ApiKeyRevoked,
            MagicLinkSent,
            MagicLinkVerified,
            OtpSent,
            OtpVerified,
            PhoneVerified,
        ]
        .into_iter()
        .collect();
        self
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    pub fn with_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    pub fn should_send(&self, event: WebhookEvent) -> bool {
        self.events.is_empty() || self.events.contains(&event)
    }
}

#[async_trait]
pub trait WebhookSender: Send + Sync {
    async fn send(&self, payload: WebhookPayload) -> Result<()>;
}

pub struct NoopWebhookSender;

#[async_trait]
impl WebhookSender for NoopWebhookSender {
    async fn send(&self, _payload: WebhookPayload) -> Result<()> {
        Ok(())
    }
}

pub struct HttpWebhookSender {
    config: WebhookConfig,
    client: reqwest::Client,
}

impl HttpWebhookSender {
    pub fn new(config: WebhookConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    fn sign_payload(&self, payload: &str, timestamp: i64) -> String {
        let message = format!("{}.{}", timestamp, payload);
        let mut mac = Hmac::<Sha256>::new_from_slice(self.config.secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(message.as_bytes());
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }
}

#[async_trait]
impl WebhookSender for HttpWebhookSender {
    async fn send(&self, payload: WebhookPayload) -> Result<()> {
        if !self.config.should_send(payload.event) {
            return Ok(());
        }

        let body =
            serde_json::to_string(&payload).map_err(|e| TsaError::Internal(e.to_string()))?;

        let timestamp = payload.timestamp.timestamp();
        let signature = self.sign_payload(&body, timestamp);

        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                let delay = std::time::Duration::from_millis(100 * 2u64.pow(attempt));
                tokio::time::sleep(delay).await;
            }

            let result = self
                .client
                .post(&self.config.url)
                .header("Content-Type", "application/json")
                .header("X-TSA-Signature", &signature)
                .header("X-TSA-Timestamp", timestamp.to_string())
                .header("X-TSA-Event", payload.event.as_str())
                .header("X-TSA-Delivery-ID", payload.id.to_string())
                .body(body.clone())
                .send()
                .await;

            match result {
                Ok(response) if response.status().is_success() => {
                    return Ok(());
                }
                Ok(response) => {
                    last_error = Some(TsaError::Internal(format!(
                        "Webhook failed with status: {}",
                        response.status()
                    )));
                }
                Err(e) => {
                    last_error = Some(TsaError::Internal(format!("Webhook request failed: {}", e)));
                }
            }
        }

        Err(last_error.unwrap_or_else(|| TsaError::Internal("Webhook failed".into())))
    }
}

pub struct AsyncWebhookSender {
    tx: mpsc::UnboundedSender<WebhookPayload>,
}

impl AsyncWebhookSender {
    pub fn new(config: WebhookConfig) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<WebhookPayload>();
        let sender = HttpWebhookSender::new(config);

        tokio::spawn(async move {
            while let Some(payload) = rx.recv().await {
                if let Err(e) = sender.send(payload).await {
                    tracing::error!("Webhook delivery failed: {}", e);
                }
            }
        });

        Self { tx }
    }
}

#[async_trait]
impl WebhookSender for AsyncWebhookSender {
    async fn send(&self, payload: WebhookPayload) -> Result<()> {
        self.tx
            .send(payload)
            .map_err(|e| TsaError::Internal(e.to_string()))
    }
}

pub fn verify_webhook_signature(
    payload: &str,
    signature: &str,
    timestamp: i64,
    secret: &str,
    tolerance_secs: i64,
) -> Result<()> {
    let now = Utc::now().timestamp();
    if (now - timestamp).abs() > tolerance_secs {
        return Err(TsaError::InvalidToken);
    }

    let message = format!("{}.{}", timestamp, payload);
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .map_err(|_| TsaError::Internal("Invalid secret".into()))?;
    mac.update(message.as_bytes());

    let expected = hex::encode(mac.finalize().into_bytes());
    if signature != expected {
        return Err(TsaError::InvalidToken);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tsa_core::{UserWebhookData, WebhookData};
    use uuid::Uuid;

    #[test]
    fn test_webhook_config() {
        let config = WebhookConfig::new("https://example.com/webhook", "secret123")
            .with_events([WebhookEvent::UserCreated, WebhookEvent::SigninSuccess])
            .with_timeout(60)
            .with_retries(5);

        assert!(config.should_send(WebhookEvent::UserCreated));
        assert!(config.should_send(WebhookEvent::SigninSuccess));
        assert!(!config.should_send(WebhookEvent::UserDeleted));
    }

    #[test]
    fn test_signature_verification() {
        let secret = "test_secret";
        let payload = r#"{"event":"user.created"}"#;
        let timestamp = Utc::now().timestamp();

        let message = format!("{}.{}", timestamp, payload);
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(message.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        assert!(verify_webhook_signature(payload, &signature, timestamp, secret, 300).is_ok());
    }

    #[test]
    fn test_signature_expired() {
        let secret = "test_secret";
        let payload = r#"{"event":"user.created"}"#;
        let timestamp = Utc::now().timestamp() - 600;

        let message = format!("{}.{}", timestamp, payload);
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(message.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        assert!(verify_webhook_signature(payload, &signature, timestamp, secret, 300).is_err());
    }
}
