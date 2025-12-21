use moka::future::Cache;
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppCache {
    pub projects: Cache<String, String>,
    pub project: Cache<String, String>,
    pub tokens: Cache<String, String>,
}

impl AppCache {
    pub fn new() -> Self {
        Self {
            projects: Cache::builder()
                .time_to_live(Duration::from_secs(300))
                .max_capacity(1000)
                .build(),
            project: Cache::builder()
                .time_to_live(Duration::from_secs(300))
                .max_capacity(5000)
                .build(),
            tokens: Cache::builder()
                .time_to_live(Duration::from_secs(300))
                .max_capacity(1000)
                .build(),
        }
    }

    pub async fn invalidate_user_projects(&self, user_id: Uuid) {
        self.projects
            .invalidate(&format!("user:{}:projects", user_id))
            .await;
    }

    pub async fn invalidate_project(&self, user_id: Uuid, slug: &str) {
        self.project
            .invalidate(&format!("user:{}:project:{}", user_id, slug))
            .await;
        self.invalidate_user_projects(user_id).await;
    }

    pub async fn invalidate_user_tokens(&self, user_id: Uuid) {
        self.tokens
            .invalidate(&format!("user:{}:tokens", user_id))
            .await;
    }
}

impl Default for AppCache {
    fn default() -> Self {
        Self::new()
    }
}
