//! Benchmarks for core ping data structures and statistical processing.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rping::stats::PingStatistics;
use std::time::Duration;

fn bench_stats_recording(c: &mut Criterion) {
    c.bench_function("stats_record_and_summary", |b| {
        b.iter(|| {
            let mut stats = PingStatistics::new(black_box("1.1.1.1"));
            for i in 1..=100 {
                stats.record_sent();
                stats.record_reply(Duration::from_millis(i));
            }
            black_box(stats.summary())
        });
    });
}

fn bench_target_resolution_literal(c: &mut Criterion) {
    c.bench_function("resolve_ipv4_literal", |b| {
        b.iter(|| {
            black_box(rping::app::resolve_target(
                black_box("192.168.1.1"),
                false,
                false,
            ))
        });
    });
}

criterion_group!(
    benches,
    bench_stats_recording,
    bench_target_resolution_literal
);
criterion_main!(benches);
