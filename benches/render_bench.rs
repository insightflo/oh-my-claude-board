use criterion::{criterion_group, criterion_main, Criterion};

fn render_benchmarks(_c: &mut Criterion) {
    // TODO: Add render benchmarks
}

criterion_group!(benches, render_benchmarks);
criterion_main!(benches);
