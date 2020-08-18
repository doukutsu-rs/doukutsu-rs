use criterion::{black_box, Criterion, criterion_group, criterion_main};
use fpa::I23F9;
fn criterion_benchmark(c: &mut Criterion) {
    //c.bench_function("fpa lib", |b| b.iter(|| fpa_lib()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
