use criterion::{criterion_group, criterion_main, Criterion};
use mnk_vmf::{
    types::Color,
    util::{stream, tokenize},
    Parser,
};

fn bench_color(c: &mut Criterion) {
    let color = tokenize(r#""color" "10 100 250""#);

    c.bench_function("parse color", |b| {
        b.iter_batched(
            || stream(color.clone()),
            |input| {
                Color::parse(input).unwrap();
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(color_benches, bench_color);
criterion_main!(color_benches);
