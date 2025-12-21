use async_graphql::ID;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct Project {
    id: ID,
    slug: String,
    name: String,
    description: Option<String>,
    public: bool,
    github_repo: Option<String>,
    github_pr_comments: bool,
    github_status_checks: bool,
    has_github_token: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ApiToken {
    id: ID,
    name: String,
    last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
}

fn create_sample_project() -> Project {
    Project {
        id: ID("550e8400-e29b-41d4-a716-446655440000".to_string()),
        slug: "my-project".to_string(),
        name: "My Project".to_string(),
        description: Some("A sample project for benchmarking".to_string()),
        public: false,
        github_repo: Some("owner/repo".to_string()),
        github_pr_comments: true,
        github_status_checks: true,
        has_github_token: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

fn create_sample_token() -> ApiToken {
    ApiToken {
        id: ID("550e8400-e29b-41d4-a716-446655440001".to_string()),
        name: "CI Token".to_string(),
        last_used_at: Some(chrono::Utc::now()),
        created_at: chrono::Utc::now(),
    }
}

fn bench_serialize_project(c: &mut Criterion) {
    let project = create_sample_project();

    c.bench_function("serialize_project", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&project)).unwrap();
            black_box(json)
        });
    });
}

fn bench_deserialize_project(c: &mut Criterion) {
    let project = create_sample_project();
    let json = serde_json::to_string(&project).unwrap();

    c.bench_function("deserialize_project", |b| {
        b.iter(|| {
            let p: Project = serde_json::from_str(black_box(&json)).unwrap();
            black_box(p)
        });
    });
}

fn bench_serialize_project_list(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialize_project_list");

    for count in [1, 10, 50, 100].iter() {
        let projects: Vec<Project> = (0..*count)
            .map(|i| {
                let mut p = create_sample_project();
                p.slug = format!("project-{}", i);
                p
            })
            .collect();

        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, _| {
            b.iter(|| {
                let json = serde_json::to_string(black_box(&projects)).unwrap();
                black_box(json)
            });
        });
    }
    group.finish();
}

fn bench_deserialize_project_list(c: &mut Criterion) {
    let mut group = c.benchmark_group("deserialize_project_list");

    for count in [1, 10, 50, 100].iter() {
        let projects: Vec<Project> = (0..*count)
            .map(|i| {
                let mut p = create_sample_project();
                p.slug = format!("project-{}", i);
                p
            })
            .collect();
        let json = serde_json::to_string(&projects).unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(count), &json, |b, json| {
            b.iter(|| {
                let p: Vec<Project> = serde_json::from_str(black_box(json)).unwrap();
                black_box(p)
            });
        });
    }
    group.finish();
}

fn bench_serialize_token(c: &mut Criterion) {
    let token = create_sample_token();

    c.bench_function("serialize_token", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&token)).unwrap();
            black_box(json)
        });
    });
}

fn bench_deserialize_token(c: &mut Criterion) {
    let token = create_sample_token();
    let json = serde_json::to_string(&token).unwrap();

    c.bench_function("deserialize_token", |b| {
        b.iter(|| {
            let t: ApiToken = serde_json::from_str(black_box(&json)).unwrap();
            black_box(t)
        });
    });
}

fn bench_roundtrip_project(c: &mut Criterion) {
    let project = create_sample_project();

    c.bench_function("roundtrip_project", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&project)).unwrap();
            let p: Project = serde_json::from_str(&json).unwrap();
            black_box(p)
        });
    });
}

criterion_group!(
    benches,
    bench_serialize_project,
    bench_deserialize_project,
    bench_serialize_project_list,
    bench_deserialize_project_list,
    bench_serialize_token,
    bench_deserialize_token,
    bench_roundtrip_project,
);
criterion_main!(benches);
