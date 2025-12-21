use anyhow::{anyhow, Context, Result};
use clap::Subcommand;
use driftwatch_api::grpc::auth::auth_service_client::AuthServiceClient;
use driftwatch_api::grpc::auth::GetMeRequest;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener as TokioTcpListener;
use tokio::sync::oneshot;
use tonic::transport::Channel;

use crate::api::{Config, DEFAULT_API_URL, DEFAULT_GRPC_URL};

#[derive(Subcommand)]
pub enum AuthCommands {
    /// Authenticate with Driftwatch
    Login {
        /// API token (skip browser auth)
        #[arg(long)]
        token: Option<String>,

        /// API URL (for self-hosted instances)
        #[arg(long, default_value = DEFAULT_API_URL)]
        api_url: String,

        /// gRPC URL (for self-hosted instances)
        #[arg(long, env = "DRIFTWATCH_GRPC_URL", default_value = DEFAULT_GRPC_URL)]
        grpc_url: String,
    },
    /// Show authentication status
    Status,
    /// Remove stored credentials
    Logout,
}

pub async fn handle(command: AuthCommands) -> Result<()> {
    match command {
        AuthCommands::Login {
            token,
            api_url,
            grpc_url,
        } => login(token, &api_url, &grpc_url).await,
        AuthCommands::Status => status().await,
        AuthCommands::Logout => logout().await,
    }
}

async fn login(token: Option<String>, api_url: &str, grpc_url: &str) -> Result<()> {
    let token = match token {
        Some(t) => {
            println!("Using provided API token...");
            t
        }
        None => {
            println!("Opening browser for authentication...");
            browser_login(api_url).await?
        }
    };

    println!("Validating token via gRPC...");
    let channel = Channel::from_shared(grpc_url.to_string())
        .context("Invalid gRPC URL")?
        .connect()
        .await
        .context("Failed to connect to gRPC server")?;

    let mut client = AuthServiceClient::new(channel);

    let response = client
        .get_me(GetMeRequest {
            token: token.clone(),
        })
        .await
        .context("Token validation failed")?;

    let user = response
        .into_inner()
        .user
        .ok_or_else(|| anyhow!("No user returned"))?;

    let config = Config {
        token,
        api_url: api_url.to_string(),
        grpc_url: grpc_url.to_string(),
    };
    let config_path = get_config_path()?;

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let config_str = toml::to_string_pretty(&config)?;
    fs::write(&config_path, config_str)?;

    println!();
    println!("Authenticated as: {}", user.email);
    println!("Config saved to: {:?}", config_path);

    Ok(())
}

async fn browser_login(api_url: &str) -> Result<String> {
    let listener = TokioTcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let callback_url = format!("http://127.0.0.1:{}/callback", port);

    // Create a channel to receive the token
    let (tx, rx) = oneshot::channel::<String>();
    let tx = Arc::new(tokio::sync::Mutex::new(Some(tx)));

    // Build the auth URL
    let auth_url = format!(
        "{}/cli-auth?callback={}",
        api_url,
        urlencoding::encode(&callback_url)
    );

    println!();
    println!("If the browser doesn't open, visit this URL:");
    println!("{}", auth_url);
    println!();

    // Open browser
    if let Err(e) = open::that(&auth_url) {
        eprintln!("Failed to open browser: {}", e);
    }

    println!("Waiting for authentication...");

    // Start local server to receive callback
    let server_handle = tokio::spawn(async move {
        loop {
            let (stream, _) = listener.accept().await.expect("Failed to accept");
            let io = TokioIo::new(stream);
            let tx = tx.clone();

            tokio::spawn(async move {
                let service = service_fn(move |req: Request<hyper::body::Incoming>| {
                    let tx = tx.clone();
                    async move { handle_callback(req, tx).await }
                });

                if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
                    eprintln!("Server error: {}", e);
                }
            });
        }
    });

    // Wait for token with timeout
    let token = tokio::time::timeout(std::time::Duration::from_secs(300), rx)
        .await
        .context("Authentication timed out (5 minutes)")?
        .context("Failed to receive token")?;

    server_handle.abort();

    Ok(token)
}

async fn handle_callback(
    req: Request<hyper::body::Incoming>,
    tx: Arc<tokio::sync::Mutex<Option<oneshot::Sender<String>>>>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let uri = req.uri();

    if uri.path() == "/callback" {
        if let Some(query) = uri.query() {
            for pair in query.split('&') {
                let mut parts = pair.splitn(2, '=');
                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                    if key == "token" {
                        let token = urlencoding::decode(value).unwrap_or_default().to_string();

                        if let Some(sender) = tx.lock().await.take() {
                            let _ = sender.send(token);
                        }

                        let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Authentication Successful</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
        }
        .container {
            text-align: center;
            padding: 2rem;
            background: rgba(255,255,255,0.1);
            border-radius: 1rem;
            backdrop-filter: blur(10px);
        }
        h1 { margin-bottom: 0.5rem; }
        p { opacity: 0.9; }
    </style>
</head>
<body>
    <div class="container">
        <h1>Authentication Successful!</h1>
        <p>You can close this window and return to your terminal.</p>
    </div>
</body>
</html>
"#;
                        return Ok(Response::builder()
                            .status(StatusCode::OK)
                            .header("Content-Type", "text/html")
                            .body(Full::new(Bytes::from(html)))
                            .unwrap());
                    }
                }
            }
        }
    }

    Ok(Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Full::new(Bytes::from("Invalid request")))
        .unwrap())
}

async fn status() -> Result<()> {
    match Config::load() {
        Ok(config) => {
            println!("Authenticated");
            println!("API URL: {}", config.api_url);
            println!("gRPC URL: {}", config.grpc_url);
            println!("Token: {}...", &config.token[..8.min(config.token.len())]);
        }
        Err(_) => {
            println!("Not authenticated");
            println!();
            println!("Run 'driftwatch auth login' to authenticate via browser");
            println!("Or 'driftwatch auth login --token <token>' to use an API token");
        }
    }
    Ok(())
}

async fn logout() -> Result<()> {
    let config_path = get_config_path()?;

    if config_path.exists() {
        fs::remove_file(&config_path)?;
        println!("Logged out successfully");
    } else {
        println!("Not logged in");
    }

    Ok(())
}

fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Could not determine config directory")?
        .join("driftwatch");
    Ok(config_dir.join("config.toml"))
}
