//! Ingest + query throughput benchmark.
//!
//! Two measurements: how fast points ingest into the time-partitioned store,
//! and how fast a windowed downsampling query folds a populated range. Both
//! scale with data size; the bench shows the shape.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

use chronodb_core::{Aggregation, DataPoint, QuerySpec, Store};

/// Builds a store of `count` points in one series, spaced 10 ticks apart.
fn populated_store(count: u64) -> Store {
    let mut store = Store::new(3600);
    for i in 0..count {
        store.ingest("cpu", DataPoint::new(i * 10, i as f64));
    }
    store
}

fn bench_ingest(c: &mut Criterion) {
    let mut group = c.benchmark_group("ingest");
    for &count in &[1_000u64, 10_000, 100_000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &n| {
            b.iter(|| black_box(populated_store(n).point_count("cpu")));
        });
    }
    group.finish();
}

fn bench_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("windowed_query");
    for &count in &[1_000u64, 10_000, 100_000] {
        let store = populated_store(count);
        let to = count * 10;
        let spec = QuerySpec::new(0, to, 300, Aggregation::Avg);
        group.bench_with_input(BenchmarkId::from_parameter(count), &spec, |b, s| {
            b.iter(|| {
                let windows = store.query("cpu", black_box(s)).unwrap();
                black_box(windows.len())
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_ingest, bench_query);
criterion_main!(benches);
