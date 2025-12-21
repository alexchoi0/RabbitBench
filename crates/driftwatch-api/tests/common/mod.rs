use async_graphql::dataloader::DataLoader;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::State,
    http::{header::AUTHORIZATION, Method, StatusCode},
    routing::{get, post},
    Router,
};
use driftwatch_api::{
    auth::{validate_token, TsaAuth},
    cache::AppCache,
    graphql::build_schema,
    loaders::{
        BenchmarkLoader, BranchLoader, MeasureLoader, MetricLoader, TestbedLoader, ThresholdLoader,
    },
    migrations,
};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection};
use serde::Deserialize;
use std::fs::{self, OpenOptions};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::oneshot;
use tower_http::cors::{Any, CorsLayer};
use tsa::{Auth, AuthConfig, NoopCallbacks};
use tsa_adapter_seaorm::SeaOrmAdapter;
use uuid::Uuid;

const CONTAINER_NAME: &str = "driftwatch-shared-test-postgres";

static INITIALIZED: AtomicBool = AtomicBool::new(false);

fn lock_file_path() -> PathBuf {
    std::env::temp_dir().join("driftwatch-test-postgres.lock")
}

fn url_file_path() -> PathBuf {
    std::env::temp_dir().join("driftwatch-test-postgres.url")
}

fn with_lock<T, F: FnOnce() -> T>(f: F) -> T {
    use fs2::FileExt;
    let lock_file = OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .open(lock_file_path())
        .expect("Failed to open lock file");
    lock_file.lock_exclusive().expect("Failed to acquire lock");
    let result = f();
    lock_file.unlock().expect("Failed to release lock");
    result
}

fn get_or_start_container() -> String {
    with_lock(|| {
        let running = std::process::Command::new("docker")
            .args(["inspect", "-f", "{{.State.Running}}", CONTAINER_NAME])
            .output()
            .map(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).trim() == "true")
            .unwrap_or(false);

        if running {
            if let Ok(url) = fs::read_to_string(url_file_path()) {
                if !url.trim().is_empty() {
                    increment_ref_count();
                    return url.trim().to_string();
                }
            }
            let port = get_container_port().unwrap_or_default();
            if !port.is_empty() {
                let url = format!(
                    "postgres://postgres:postgres@localhost:{}/postgres?sslmode=disable",
                    port
                );
                let _ = fs::write(url_file_path(), &url);
                increment_ref_count();
                return url;
            }
        }

        let _ = std::process::Command::new("docker")
            .args(["rm", "-f", CONTAINER_NAME])
            .output();

        let output = std::process::Command::new("docker")
            .args([
                "run",
                "-d",
                "--name",
                CONTAINER_NAME,
                "-e",
                "POSTGRES_USER=postgres",
                "-e",
                "POSTGRES_PASSWORD=postgres",
                "-e",
                "POSTGRES_DB=postgres",
                "-p",
                "0:5432",
                "postgres:16-alpine",
            ])
            .output()
            .expect("Failed to start container");

        if !output.status.success() {
            panic!(
                "Failed to start postgres container: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        for _ in 0..100 {
            let ready = std::process::Command::new("docker")
                .args(["exec", CONTAINER_NAME, "pg_isready", "-U", "postgres"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            if ready {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        std::thread::sleep(std::time::Duration::from_millis(200));

        let port = get_container_port().expect("Failed to get container port");
        let url = format!(
            "postgres://postgres:postgres@localhost:{}/postgres?sslmode=disable",
            port
        );
        let _ = fs::write(url_file_path(), &url);
        let _ = fs::write(ref_count_path(), "1");

        url
    })
}

fn get_container_port() -> Option<String> {
    let output = std::process::Command::new("docker")
        .args(["port", CONTAINER_NAME, "5432"])
        .output()
        .ok()?;
    let port_str = String::from_utf8_lossy(&output.stdout);
    port_str.trim().split(':').last().map(|s| s.to_string())
}

fn ref_count_path() -> PathBuf {
    std::env::temp_dir().join("driftwatch-test-postgres.refcount")
}

fn increment_ref_count() {
    let path = ref_count_path();
    let count: u32 = fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);
    let _ = fs::write(&path, (count + 1).to_string());
}

fn decrement_ref_count() -> u32 {
    let path = ref_count_path();
    let count: u32 = fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(1);
    let new_count = count.saturating_sub(1);
    let _ = fs::write(&path, new_count.to_string());
    new_count
}

#[ctor::ctor]
fn init() {
    if std::env::var("TEST_DATABASE_URL").is_ok() {
        return;
    }

    let url = get_or_start_container();
    std::env::set_var("TEST_DATABASE_URL", &url);
    INITIALIZED.store(true, Ordering::SeqCst);
}

#[ctor::dtor]
fn cleanup() {
    if !INITIALIZED.load(Ordering::SeqCst) {
        return;
    }

    with_lock(|| {
        let remaining = decrement_ref_count();

        if remaining == 0 {
            let _ = std::process::Command::new("docker")
                .args(["rm", "-f", CONTAINER_NAME])
                .output();
            let _ = fs::remove_file(url_file_path());
            let _ = fs::remove_file(ref_count_path());
        }
    });
}

fn get_postgres_url() -> String {
    std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL not set - ctor should have set it")
}

type AppSchema = async_graphql::Schema<
    driftwatch_api::graphql::QueryRoot,
    driftwatch_api::graphql::MutationRoot,
    async_graphql::EmptySubscription,
>;

#[derive(Clone)]
struct TestAppState {
    schema: AppSchema,
    db: DatabaseConnection,
    auth: Arc<TsaAuth>,
    cache: AppCache,
}

async fn graphql_handler(
    State(state): State<TestAppState>,
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
            Err(_) => None,
        },
        None => None,
    };

    let mut request = req.into_inner();
    request = request.data(state.db.clone());
    request = request.data(state.cache.clone());
    request = request.data(state.auth.clone());

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

pub struct TestServer {
    pub base_url: String,
    pub client: reqwest::Client,
    auth: Arc<TsaAuth>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    db_name: String,
    admin_url: String,
}

impl TestServer {
    pub async fn new() -> Self {
        let admin_url = get_postgres_url();
        let db_name = format!("test_{}", Uuid::new_v4().to_string().replace('-', "_"));

        let admin_db = Database::connect(&admin_url)
            .await
            .expect("Connect to admin db");
        admin_db
            .execute_unprepared(&format!("CREATE DATABASE \"{}\"", db_name))
            .await
            .expect("Create test database");
        drop(admin_db);

        let test_url = if let Some(idx) = admin_url.rfind('/') {
            format!("{}/{}", &admin_url[..idx], db_name)
        } else {
            format!("{}/{}", admin_url, db_name)
        };
        let db = Database::connect(&test_url)
            .await
            .expect("Connect to test db");

        migrations::run_migrations(&db)
            .await
            .expect("Run migrations");

        let adapter = SeaOrmAdapter::new(db.clone());
        let auth_config = AuthConfig::new().app_name("Driftwatch Test");
        let auth = Arc::new(Auth::new(adapter, auth_config, NoopCallbacks));

        let port = portpicker::pick_unused_port().expect("No available port");

        let schema = build_schema();
        let cache = AppCache::new();

        let state = TestAppState {
            schema,
            db,
            auth: auth.clone(),
            cache,
        };

        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers(Any);

        let app = Router::new()
            .route("/health", get(|| async { "OK" }))
            .route("/graphql", post(graphql_handler))
            .layer(cors)
            .with_state(state);

        let addr = format!("127.0.0.1:{}", port);
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .expect("Bind to port");

        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await
                .ok();
        });

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        Self {
            base_url: format!("http://127.0.0.1:{}", port),
            client: reqwest::Client::new(),
            auth,
            shutdown_tx: Some(shutdown_tx),
            db_name,
            admin_url,
        }
    }

    pub async fn create_test_user(&self, email: &str) -> String {
        let password = "test_password_123!";
        let (_, _, token) = self
            .auth
            .signup(email, password, Some(email.to_string()))
            .await
            .expect("Create test user");
        token
    }

    pub fn create_test_token(&self, user_id: &str) -> String {
        let rt = tokio::runtime::Handle::current();
        let auth = self.auth.clone();
        let email = format!("{}@test.local", user_id);

        std::thread::scope(|s| {
            s.spawn(|| {
                rt.block_on(async {
                    let password = "test_password_123!";
                    match auth
                        .signup(&email, password, Some(user_id.to_string()))
                        .await
                    {
                        Ok((_, _, token)) => token,
                        Err(_) => {
                            let (_, _, token) = auth
                                .signin(&email, password, None, None)
                                .await
                                .expect("Signin existing user");
                            token
                        }
                    }
                })
            })
            .join()
            .expect("Thread panicked")
        })
    }

    pub async fn graphql<T: for<'de> Deserialize<'de>>(
        &self,
        query: &str,
        variables: Option<serde_json::Value>,
        token: Option<&str>,
    ) -> GraphQLResult<T> {
        let mut req =
            self.client
                .post(format!("{}/graphql", self.base_url))
                .json(&serde_json::json!({
                    "query": query,
                    "variables": variables.unwrap_or(serde_json::Value::Null)
                }));

        if let Some(token) = token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        let response = req.send().await.expect("Send GraphQL request");
        let body: serde_json::Value = response.json().await.expect("Parse response");

        if let Some(errors) = body.get("errors") {
            return GraphQLResult {
                data: None,
                errors: Some(errors.clone()),
            };
        }

        let data: T = serde_json::from_value(body["data"].clone()).expect("Parse data");
        GraphQLResult {
            data: Some(data),
            errors: None,
        }
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }

        let admin_url = self.admin_url.clone();
        let db_name = self.db_name.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if let Ok(admin_db) = Database::connect(&admin_url).await {
                    let _ = admin_db
                        .execute_unprepared(&format!(
                            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}'",
                            db_name
                        ))
                        .await;
                    let _ = admin_db
                        .execute_unprepared(&format!("DROP DATABASE IF EXISTS \"{}\"", db_name))
                        .await;
                }
            });
        });
    }
}

pub struct GraphQLResult<T> {
    pub data: Option<T>,
    pub errors: Option<serde_json::Value>,
}

impl<T> GraphQLResult<T> {
    pub fn unwrap(self) -> T {
        if let Some(errors) = self.errors {
            panic!("GraphQL errors: {:?}", errors);
        }
        self.data.expect("No data in response")
    }

    pub fn expect_error(self) -> serde_json::Value {
        self.errors.expect("Expected error but got success")
    }
}

#[macro_export]
macro_rules! test_server {
    () => {
        common::TestServer::new().await
    };
}
