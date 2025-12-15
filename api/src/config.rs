use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub better_auth_secret: String,
    pub port: u16,
    pub rust_log: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            better_auth_secret: env::var("BETTER_AUTH_SECRET")
                .expect("BETTER_AUTH_SECRET must be set"),
            port: env::var("PORT")
                .unwrap_or_else(|_| "4000".to_string())
                .parse()
                .expect("PORT must be a valid number"),
            rust_log: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        }
    }
}
