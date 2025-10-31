use criterion::{criterion_group, criterion_main, Criterion};
use vmf::{
    types::Entity,
    util::{stream, tokenize},
    Parser,
};

fn bench_entity(c: &mut Criterion) {
    let input_str = r#"entity
        {
            "id" "85"
            "classname" "light"
            "_light" "255 255 255 400"
            "_lightHDR" "-1 -1 -1 1"
            "_lightscaleHDR" "1"
            "_quadratic_attn" "1"
            "origin" "-192 192 128"
            editor
            {
                "color" "220 30 220"
                "visgroupshown" "1"
                "visgroupautoshown" "1"
                "logicalpos" "[0 3500]"
            }
        }"#;

    // Validate the input parses correctly before benchmarking
    let test_tokens = tokenize(input_str);
    Entity::parse(stream(test_tokens)).expect("Benchmark input should be valid Entity");

    let entity = tokenize(input_str);

    c.bench_function("parse entity", |b| {
        b.iter_batched(
            || stream(entity.clone()),
            |input| {
                Entity::parse(input).unwrap();
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(entity_benches, bench_entity);
criterion_main!(entity_benches);
