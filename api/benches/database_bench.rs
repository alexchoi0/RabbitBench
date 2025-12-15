use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use driftwatch_api::entities::{self, project, branch};
use pgwire_yachtsql::{ServerConfig, YachtServer};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, Database, DatabaseConnection, EntityTrait};
use tokio::runtime::Runtime;
use uuid::Uuid;

const SCHEMA: &str = r#"
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT,
    public BOOLEAN NOT NULL DEFAULT false,
    github_repo TEXT,
    github_token TEXT,
    github_pr_comments BOOLEAN NOT NULL DEFAULT false,
    github_status_checks BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE branches (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE testbeds (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE measures (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    name TEXT NOT NULL,
    unit TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE benchmarks (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE reports (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    branch_id TEXT NOT NULL,
    testbed_id TEXT NOT NULL,
    git_hash TEXT,
    pr_number INT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE metrics (
    id TEXT PRIMARY KEY,
    report_id TEXT NOT NULL,
    benchmark_id TEXT NOT NULL,
    measure_id TEXT NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    lower DOUBLE PRECISION,
    upper DOUBLE PRECISION,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
"#;

struct BenchContext {
    rt: Runtime,
    server: Option<YachtServer>,
    db: Option<DatabaseConnection>,
}

impl BenchContext {
    fn new() -> Self {
        let rt = Runtime::new().unwrap();
        let (server, db) = rt.block_on(async {
            let server = YachtServer::start(ServerConfig {
                port: 0,
                schema_sql: SCHEMA.to_string(),
            })
            .await
            .unwrap();
            let db = Database::connect(server.connection_url()).await.unwrap();
            (server, db)
        });
        Self {
            rt,
            server: Some(server),
            db: Some(db),
        }
    }

    fn db(&self) -> &DatabaseConnection {
        self.db.as_ref().unwrap()
    }

    fn runtime(&self) -> &Runtime {
        &self.rt
    }
}

impl Drop for BenchContext {
    fn drop(&mut self) {
        if let Some(server) = self.server.take() {
            self.rt.block_on(async {
                server.shutdown().await;
            });
        }
    }
}

fn bench_insert_project(c: &mut Criterion) {
    let ctx = BenchContext::new();
    let mut counter = 0u64;

    c.bench_function("insert_project", |b| {
        b.to_async(ctx.runtime()).iter(|| {
            counter += 1;
            let db = ctx.db().clone();
            async move {
                let now = chrono::Utc::now().fixed_offset();
                let project = project::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    user_id: Set("bench-user".to_string()),
                    slug: Set(format!("bench-project-{}", counter)),
                    name: Set("Benchmark Project".to_string()),
                    description: Set(Some("A benchmark project".to_string())),
                    public: Set(false),
                    github_repo: Set(None),
                    github_token: Set(None),
                    github_pr_comments: Set(false),
                    github_status_checks: Set(false),
                    created_at: Set(now),
                    updated_at: Set(now),
                };
                let result = project.insert(&db).await;
                black_box(result)
            }
        });
    });
}

fn bench_select_project_by_id(c: &mut Criterion) {
    let ctx = BenchContext::new();

    // Insert a project to query
    let project_id = ctx.runtime().block_on(async {
        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::new_v4();
        let project = project::ActiveModel {
            id: Set(id),
            user_id: Set("bench-user".to_string()),
            slug: Set("select-bench".to_string()),
            name: Set("Select Benchmark".to_string()),
            description: Set(None),
            public: Set(false),
            github_repo: Set(None),
            github_token: Set(None),
            github_pr_comments: Set(false),
            github_status_checks: Set(false),
            created_at: Set(now),
            updated_at: Set(now),
        };
        project.insert(ctx.db()).await.unwrap();
        id
    });

    c.bench_function("select_project_by_id", |b| {
        b.to_async(ctx.runtime()).iter(|| {
            let db = ctx.db().clone();
            let id = project_id;
            async move {
                let result = entities::Project::find_by_id(id).one(&db).await;
                black_box(result)
            }
        });
    });
}

fn bench_select_all_projects(c: &mut Criterion) {
    let mut group = c.benchmark_group("select_all_projects");

    for count in [1, 10, 50, 100].iter() {
        let ctx = BenchContext::new();

        // Insert projects
        ctx.runtime().block_on(async {
            let now = chrono::Utc::now().fixed_offset();
            for i in 0..*count {
                let project = project::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    user_id: Set("bench-user".to_string()),
                    slug: Set(format!("project-{}", i)),
                    name: Set(format!("Project {}", i)),
                    description: Set(None),
                    public: Set(false),
                    github_repo: Set(None),
                    github_token: Set(None),
                    github_pr_comments: Set(false),
                    github_status_checks: Set(false),
                    created_at: Set(now),
                    updated_at: Set(now),
                };
                project.insert(ctx.db()).await.unwrap();
            }
        });

        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, _| {
            b.to_async(ctx.runtime()).iter(|| {
                let db = ctx.db().clone();
                async move {
                    let result = entities::Project::find().all(&db).await;
                    black_box(result)
                }
            });
        });
    }
    group.finish();
}

fn bench_insert_with_relation(c: &mut Criterion) {
    let ctx = BenchContext::new();

    // Insert a project first
    let project_id = ctx.runtime().block_on(async {
        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::new_v4();
        let project = project::ActiveModel {
            id: Set(id),
            user_id: Set("bench-user".to_string()),
            slug: Set("relation-bench".to_string()),
            name: Set("Relation Benchmark".to_string()),
            description: Set(None),
            public: Set(false),
            github_repo: Set(None),
            github_token: Set(None),
            github_pr_comments: Set(false),
            github_status_checks: Set(false),
            created_at: Set(now),
            updated_at: Set(now),
        };
        project.insert(ctx.db()).await.unwrap();
        id
    });

    let mut counter = 0u64;

    c.bench_function("insert_branch", |b| {
        b.to_async(ctx.runtime()).iter(|| {
            counter += 1;
            let db = ctx.db().clone();
            let pid = project_id;
            async move {
                let now = chrono::Utc::now().fixed_offset();
                let branch = branch::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    project_id: Set(pid),
                    name: Set(format!("branch-{}", counter)),
                    created_at: Set(now),
                    updated_at: Set(now),
                };
                let result = branch.insert(&db).await;
                black_box(result)
            }
        });
    });
}

criterion_group!(
    benches,
    bench_insert_project,
    bench_select_project_by_id,
    bench_select_all_projects,
    bench_insert_with_relation,
);
criterion_main!(benches);
