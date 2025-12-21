use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub grpc_port: u16,
    pub rust_log: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            port: env::var("PORT")
                .unwrap_or_else(|_| "4000".to_string())
                .parse()
                .expect("PORT must be a valid number"),
            grpc_port: env::var("GRPC_PORT")
                .unwrap_or_else(|_| "50051".to_string())
                .parse()
                .expect("GRPC_PORT must be a valid number"),
            rust_log: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        }
    }
}
