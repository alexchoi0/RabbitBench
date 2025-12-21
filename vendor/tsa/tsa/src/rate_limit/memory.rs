use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Instant;
use tsa_core::Result;

use super::{RateLimitConfig, RateLimitResult, RateLimiter};

struct RateLimitEntry {
    count: u32,
    window_start: Instant,
}

pub struct InMemoryRateLimiter {
    entries: RwLock<HashMap<String, RateLimitEntry>>,
}

impl InMemoryRateLimiter {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }

    fn cleanup_expired(&self) {
        let mut entries = self.entries.write().unwrap();
        let now = Instant::now();
        entries.retain(|_, entry| {
            now.duration_since(entry.window_start) < std::time::Duration::from_secs(3600)
        });
    }
}

impl Default for InMemoryRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RateLimiter for InMemoryRateLimiter {
    async fn check(&self, key: &str, config: &RateLimitConfig) -> Result<RateLimitResult> {
        let now = Instant::now();

        let mut entries = self.entries.write().unwrap();

        let entry = entries.entry(key.to_string()).or_insert(RateLimitEntry {
            count: 0,
            window_start: now,
        });

        if now.duration_since(entry.window_start) >= config.window {
            entry.count = 0;
            entry.window_start = now;
        }

        let allowed = entry.count < config.max_requests;
        let remaining = if allowed {
            config.max_requests - entry.count - 1
        } else {
            0
        };

        if allowed {
            entry.count += 1;
        }

        let reset_at = entry.window_start + config.window;

        Ok(RateLimitResult {
            allowed,
            remaining,
            reset_at,
        })
    }

    async fn reset(&self, key: &str) -> Result<()> {
        let mut entries = self.entries.write().unwrap();
        entries.remove(key);
        Ok(())
    }
}

impl InMemoryRateLimiter {
    pub fn start_cleanup_task(self: std::sync::Arc<Self>, interval: std::time::Duration) {
        let limiter = self;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            loop {
                interval.tick().await;
                limiter.cleanup_expired();
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_rate_limiting() {
        let limiter = InMemoryRateLimiter::new();
        let config = RateLimitConfig::new(3, Duration::from_secs(60));

        let result = limiter.check("test_key", &config).await.unwrap();
        assert!(result.allowed);
        assert_eq!(result.remaining, 2);

        let result = limiter.check("test_key", &config).await.unwrap();
        assert!(result.allowed);
        assert_eq!(result.remaining, 1);

        let result = limiter.check("test_key", &config).await.unwrap();
        assert!(result.allowed);
        assert_eq!(result.remaining, 0);

        let result = limiter.check("test_key", &config).await.unwrap();
        assert!(!result.allowed);
        assert_eq!(result.remaining, 0);
    }

    #[tokio::test]
    async fn test_rate_limit_reset() {
        let limiter = InMemoryRateLimiter::new();
        let config = RateLimitConfig::new(1, Duration::from_secs(60));

        let result = limiter.check("test_key", &config).await.unwrap();
        assert!(result.allowed);

        let result = limiter.check("test_key", &config).await.unwrap();
        assert!(!result.allowed);

        limiter.reset("test_key").await.unwrap();

        let result = limiter.check("test_key", &config).await.unwrap();
        assert!(result.allowed);
    }
}
