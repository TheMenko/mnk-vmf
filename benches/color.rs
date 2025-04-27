use criterion::{criterion_group, criterion_main, Criterion};
use vmf::{types::Color, Parser};

fn bench_color(c: &mut Criterion) {
    let color = r#""color" "10 100 250""#;

    c.bench_function("parse color", |b| {
        b.iter(|| {
            Color::parse(color).unwrap();
        })
    });
}

criterion_group!(color_benches, bench_color);
criterion_main!(color_benches);
