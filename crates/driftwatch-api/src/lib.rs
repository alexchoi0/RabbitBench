pub mod auth;
pub mod cache;
pub mod config;
pub mod entities;
pub mod graphql;
pub mod grpc;
pub mod loaders;
pub mod migrations;

use std::sync::Arc;

use async_graphql::dataloader::DataLoader;
use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::State,
    http::{header::AUTHORIZATION, Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use sea_orm::{Database, DatabaseConnection};
use tonic::transport::Server as TonicServer;
use tsa::{Auth, AuthConfig, NoopCallbacks};
use tsa_adapter_seaorm::SeaOrmAdapter;

use auth::{validate_token, TsaAuth};
use cache::AppCache;
use grpc::auth::auth_service_server::AuthServiceServer;
use grpc::AuthServiceImpl;
use loaders::{
    BenchmarkLoader, BranchLoader, MeasureLoader, MetricLoader, TestbedLoader, ThresholdLoader,
};
use tower_http::cors::{Any, CorsLayer};

use config::Config;
use graphql::{build_schema, AppSchema};

#[derive(Clone)]
struct AppState {
    schema: AppSchema,
    db: DatabaseConnection,
    auth: Arc<TsaAuth>,
    auth_service: Arc<AuthServiceImpl>,
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
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    let user = match auth_header {
        Some(token) => match validate_token(token, &state.auth).await {
            Ok(user) => Some(user),
            Err(e) => {
                tracing::warn!("Token validation failed: {}", e.0);
                None
            }
        },
        None => None,
    };

    let mut request = req.into_inner();
    request = request.data(state.db.clone());
    request = request.data(state.cache.clone());
    request = request.data(state.auth.clone());
    request = request.data(state.auth_service.clone());

    request = request.data(DataLoader::new(
        BranchLoader {
            db: state.db.clone(),
        },
        tokio::spawn,
    ));
    request = request.data(DataLoader::new(
        TestbedLoader {
            db: state.db.clone(),
        },
        tokio::spawn,
    ));
    request = request.data(DataLoader::new(
        BenchmarkLoader {
            db: state.db.clone(),
        },
        tokio::spawn,
    ));
    request = request.data(DataLoader::new(
        MeasureLoader {
            db: state.db.clone(),
        },
        tokio::spawn,
    ));
    request = request.data(DataLoader::new(
        MetricLoader {
            db: state.db.clone(),
        },
        tokio::spawn,
    ));
    request = request.data(DataLoader::new(
        ThresholdLoader {
            db: state.db.clone(),
        },
        tokio::spawn,
    ));

    if let Some(user) = user {
        request = request.data(user);
    }

    Ok(state.schema.execute(request).await.into())
}

pub async fn serve(port: Option<u16>, grpc_port: Option<u16>) -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let mut config = Config::from_env();
    if let Some(p) = port {
        config.port = p;
    }

    tracing::info!("Connecting to database...");

    let db = Database::connect(&config.database_url).await?;

    tracing::info!("Database connected");

    migrations::run_migrations(&db).await?;

    let adapter = SeaOrmAdapter::new(db.clone());
    let auth_config = AuthConfig::new().app_name("Driftwatch");
    let auth = Arc::new(Auth::new(adapter, auth_config, NoopCallbacks));
    let auth_service = Arc::new(AuthServiceImpl { auth: auth.clone() });

    let schema = build_schema();

    let cache = AppCache::new();
    let state = AppState {
        schema,
        db,
        auth: auth.clone(),
        auth_service: auth_service.clone(),
        cache,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health))
        .route("/graphql", post(graphql_handler))
        .route("/graphiql", get(graphiql))
        .layer(cors)
        .with_state(state);

    let grpc_port = grpc_port.unwrap_or(config.grpc_port);
    let grpc_addr = format!("0.0.0.0:{}", grpc_port).parse()?;
    let grpc_auth_service = AuthServiceImpl { auth };

    let grpc_handle = tokio::spawn(async move {
        tracing::info!("Starting gRPC server on {}", grpc_addr);
        TonicServer::builder()
            .add_service(AuthServiceServer::new(grpc_auth_service))
            .serve(grpc_addr)
            .await
    });

    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("Starting HTTP server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    let http_handle = tokio::spawn(async move { axum::serve(listener, app).await });

    tokio::select! {
        result = grpc_handle => {
            result??;
        }
        result = http_handle => {
            result??;
        }
    }

    Ok(())
}
