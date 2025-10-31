use criterion::{criterion_group, criterion_main, Criterion};
use vmf::{
    types::Cordon,
    util::{stream, tokenize},
    Parser,
};

fn bench_cordon(c: &mut Criterion) {
    let input_str = r#"cordon
        {
            "mins" "(-1024 -1024 -1024)"
            "maxs" "(1024 1024 1024)"
            "active" "0"
        }"#;

    // Validate the input parses correctly before benchmarking
    let test_tokens = tokenize(input_str);
    Cordon::parse(stream(test_tokens)).expect("Benchmark input should be valid Cordon");

    let cordon = tokenize(input_str);

    c.bench_function("parse cordon", |b| {
        b.iter_batched(
            || stream(cordon.clone()),
            |input| {
                Cordon::parse(input).unwrap();
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(cordon_benches, bench_cordon);
criterion_main!(cordon_benches);
