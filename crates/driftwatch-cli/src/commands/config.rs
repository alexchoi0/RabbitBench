use anyhow::{Context, Result};
use clap::Subcommand;
use std::fs;
use std::path::PathBuf;

use crate::api::Config;

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Set configuration values
    Set {
        /// API server URL
        #[arg(long)]
        api_url: Option<String>,

        /// gRPC server URL
        #[arg(long)]
        grpc_url: Option<String>,
    },
    /// Show current configuration
    Show,
}

pub async fn handle(command: ConfigCommands) -> Result<()> {
    match command {
        ConfigCommands::Set { api_url, grpc_url } => set(api_url, grpc_url).await,
        ConfigCommands::Show => show().await,
    }
}

async fn set(api_url: Option<String>, grpc_url: Option<String>) -> Result<()> {
    if api_url.is_none() && grpc_url.is_none() {
        println!("No configuration options provided.");
        println!("Usage: driftwatch config set --api-url <url> --grpc-url <url>");
        return Ok(());
    }

    let config_path = get_config_path()?;

    let mut config = if config_path.exists() {
        let config_str = fs::read_to_string(&config_path).context("Failed to read config file")?;
        toml::from_str(&config_str).context("Invalid config file")?
    } else {
        Config {
            token: String::new(),
            api_url: String::new(),
            grpc_url: String::new(),
        }
    };

    if let Some(url) = api_url {
        config.api_url = url.trim_end_matches('/').to_string();
        println!("API URL set to: {}", config.api_url);
    }

    if let Some(url) = grpc_url {
        config.grpc_url = url.trim_end_matches('/').to_string();
        println!("gRPC URL set to: {}", config.grpc_url);
    }

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let config_str = toml::to_string_pretty(&config)?;
    fs::write(&config_path, config_str)?;

    println!("Config saved to: {:?}", config_path);

    if config.token.is_empty() {
        println!();
        println!("Note: You are not authenticated. Run 'driftwatch auth login' to authenticate.");
    }

    Ok(())
}

async fn show() -> Result<()> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        println!("No configuration file found.");
        println!("Run 'driftwatch auth login' to authenticate and create a config.");
        return Ok(());
    }

    let config_str = fs::read_to_string(&config_path).context("Failed to read config file")?;
    let config: Config = toml::from_str(&config_str).context("Invalid config file")?;

    println!("Config file: {:?}", config_path);
    println!();
    println!("API URL: {}", config.api_url);
    println!("gRPC URL: {}", config.grpc_url);
    if !config.token.is_empty() {
        println!("Token: {}...", &config.token[..8.min(config.token.len())]);
    } else {
        println!("Token: (not set)");
    }

    Ok(())
}

fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Could not determine config directory")?
        .join("driftwatch");
    Ok(config_dir.join("config.toml"))
}
