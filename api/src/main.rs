mod auth;
mod cache;
mod config;
mod entities;
mod graphql;
mod loaders;
mod migrations;

use axum::{
    extract::State,
    http::{header::AUTHORIZATION, Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use async_graphql::dataloader::DataLoader;
use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use sea_orm::{Database, DatabaseConnection};

use cache::AppCache;
use loaders::{BenchmarkLoader, BranchLoader, MeasureLoader, MetricLoader, TestbedLoader, ThresholdLoader};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use auth::validate_token;
use config::Config;
use graphql::{build_schema, AppSchema};

#[derive(Clone)]
struct AppState {
    schema: AppSchema,
    db: DatabaseConnection,
    config: Config,
    cache: AppCache,
}

async fn health() -> &'static str {
    "OK"
}

async fn graphiql() -> impl IntoResponse {
    axum::response::Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

async fn graphql_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    req: GraphQLRequest,
) -> Result<GraphQLResponse, (StatusCode, String)> {
    // Extract and validate JWT token
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    let user = match auth_header {
        Some(token) => {
            match validate_token(token, &state.config.better_auth_secret) {
                Ok(user) => Some(user),
                Err(e) => {
                    tracing::warn!("Token validation failed: {}", e.0);
                    None
                }
            }
        }
        None => None,
    };

    // Build request with context
    let mut request = req.into_inner();
    request = request.data(state.db.clone());
    request = request.data(state.cache.clone());

    // Add DataLoaders
    request = request.data(DataLoader::new(
        BranchLoader { db: state.db.clone() },
        tokio::spawn,
    ));
    request = request.data(DataLoader::new(
        TestbedLoader { db: state.db.clone() },
        tokio::spawn,
    ));
    request = request.data(DataLoader::new(
        BenchmarkLoader { db: state.db.clone() },
        tokio::spawn,
    ));
    request = request.data(DataLoader::new(
        MeasureLoader { db: state.db.clone() },
        tokio::spawn,
    ));
    request = request.data(DataLoader::new(
        MetricLoader { db: state.db.clone() },
        tokio::spawn,
    ));
    request = request.data(DataLoader::new(
        ThresholdLoader { db: state.db.clone() },
        tokio::spawn,
    ));

    if let Some(user) = user {
        request = request.data(user);
    }

    Ok(state.schema.execute(request).await.into())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize config
    let config = Config::from_env();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| config.rust_log.clone().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Connecting to database...");

    // Connect to database
    let db = Database::connect(&config.database_url).await?;

    tracing::info!("Database connected");

    // Run migrations
    migrations::run_migrations(&db).await?;

    // Build GraphQL schema
    let schema = build_schema();

    // Create cache and app state
    let cache = AppCache::new();
    let state = AppState {
        schema,
        db,
        config: config.clone(),
        cache,
    };

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/health", get(health))
        .route("/graphql", post(graphql_handler))
        .route("/graphiql", get(graphiql))
        .layer(cors)
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
