use criterion::{black_box, criterion_group, criterion_main, Criterion};
use webb_cli::mixer::Mixer;

fn new_mixer(c: &mut Criterion) {
    c.bench_function("new mixer", |b| b.iter(|| Mixer::new(black_box(0))));
}

criterion_group!(benches, new_mixer);
criterion_main!(benches);
