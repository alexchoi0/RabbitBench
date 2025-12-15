use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use moka::future::Cache;
use std::time::Duration;
use tokio::runtime::Runtime;

fn create_cache() -> Cache<String, String> {
    Cache::builder()
        .time_to_live(Duration::from_secs(300))
        .max_capacity(1000)
        .build()
}

fn bench_cache_insert(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let cache = create_cache();

    c.bench_function("cache_insert", |b| {
        b.to_async(&rt).iter(|| async {
            let key = format!("user:{}:projects", black_box("test-user-id"));
            let value = black_box(r#"[{"id":"1","slug":"test","name":"Test"}]"#.to_string());
            cache.insert(key, value).await;
        });
    });
}

fn bench_cache_get_hit(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let cache = create_cache();

    rt.block_on(async {
        cache.insert("user:test:projects".to_string(), "cached_value".to_string()).await;
    });

    c.bench_function("cache_get_hit", |b| {
        b.to_async(&rt).iter(|| async {
            let result = cache.get(&black_box("user:test:projects".to_string())).await;
            black_box(result)
        });
    });
}

fn bench_cache_get_miss(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let cache = create_cache();

    c.bench_function("cache_get_miss", |b| {
        b.to_async(&rt).iter(|| async {
            let result = cache.get(&black_box("nonexistent_key".to_string())).await;
            black_box(result)
        });
    });
}

fn bench_cache_invalidate(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let cache = create_cache();

    c.bench_function("cache_invalidate", |b| {
        let cache = cache.clone();
        b.to_async(&rt).iter(|| {
            let cache = cache.clone();
            async move {
                cache.invalidate(&black_box("user:test:projects".to_string())).await;
            }
        });
    });
}

fn bench_cache_insert_varying_sizes(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("cache_insert_by_size");

    for size in [100, 1000, 10000, 100000].iter() {
        let value = "x".repeat(*size);
        let cache = create_cache();
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            let cache = cache.clone();
            b.to_async(&rt).iter(|| {
                let cache = cache.clone();
                let v = value.clone();
                async move {
                    cache.insert(black_box("key".to_string()), black_box(v)).await;
                }
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_cache_insert,
    bench_cache_get_hit,
    bench_cache_get_miss,
    bench_cache_invalidate,
    bench_cache_insert_varying_sizes,
);
criterion_main!(benches);
