use anyhow::{anyhow, Context, Result};
use clap::Subcommand;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

use crate::api::Config;

const DEFAULT_API_URL: &str = "https://rabbitbench.dev";

#[derive(Subcommand)]
pub enum AuthCommands {
    /// Authenticate with RabbitBench
    Login {
        /// API token (skip browser auth)
        #[arg(long)]
        token: Option<String>,

        /// API URL (for self-hosted instances)
        #[arg(long, default_value = DEFAULT_API_URL)]
        api_url: String,
    },
    /// Show authentication status
    Status,
    /// Remove stored credentials
    Logout,
}

pub async fn handle(command: AuthCommands) -> Result<()> {
    match command {
        AuthCommands::Login { token, api_url } => login(token, &api_url).await,
        AuthCommands::Status => status().await,
        AuthCommands::Logout => logout().await,
    }
}

async fn login(token: Option<String>, api_url: &str) -> Result<()> {
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

    // Validate token by making a test request
    println!("Validating token...");
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/graphql", api_url))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(r#"{"query":"{ me { email } }"}"#)
        .send()
        .await
        .context("Failed to validate token")?;

    if !resp.status().is_success() {
        return Err(anyhow!("Invalid token or server error: {}", resp.status()));
    }

    let body: serde_json::Value = resp.json().await?;
    if let Some(errors) = body.get("errors") {
        return Err(anyhow!("Token validation failed: {:?}", errors));
    }

    let email = body
        .get("data")
        .and_then(|d| d.get("me"))
        .and_then(|m| m.get("email"))
        .and_then(|e| e.as_str())
        .unwrap_or("unknown");

    // Save config
    let config = Config {
        token,
        api_url: api_url.to_string(),
    };
    let config_path = get_config_path()?;

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let config_str = toml::to_string_pretty(&config)?;
    fs::write(&config_path, config_str)?;

    println!();
    println!("Authenticated as: {}", email);
    println!("Config saved to: {:?}", config_path);

    Ok(())
}

async fn browser_login(api_url: &str) -> Result<String> {
    // Find an available port
    let listener = TcpListener::bind("127.0.0.1:0").await?;
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
            println!("Token: {}...", &config.token[..8.min(config.token.len())]);
        }
        Err(_) => {
            println!("Not authenticated");
            println!();
            println!("Run 'rabbitbench auth login' to authenticate via browser");
            println!("Or 'rabbitbench auth login --token <token>' to use an API token");
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
        .join("rabbitbench");
    Ok(config_dir.join("config.toml"))
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut result = String::new();
        for c in s.chars() {
            match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
                _ => {
                    for b in c.to_string().as_bytes() {
                        result.push_str(&format!("%{:02X}", b));
                    }
                }
            }
        }
        result
    }

    pub fn decode(s: &str) -> Result<String, std::string::FromUtf8Error> {
        let mut result = Vec::new();
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '%' {
                let hex: String = chars.by_ref().take(2).collect();
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte);
                }
            } else if c == '+' {
                result.push(b' ');
            } else {
                result.push(c as u8);
            }
        }
        String::from_utf8(result)
    }
}
