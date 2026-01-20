//! Benchmark for parser performance
//!
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nginx_discovery::parse;

fn bench_parse_simple(c: &mut Criterion) {
    let config = r#"
        user nginx;
        worker_processes auto;
    "#;

    c.bench_function("parse_simple", |b| {
        b.iter(|| {
            let _ = parse(black_box(config));
        });
    });
}

criterion_group!(benches, bench_parse_simple);
criterion_main!(benches);
