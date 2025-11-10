use criterion::{Criterion, criterion_group, criterion_main};
use mnk_vmf::{
    Parser,
    types::Side,
    util::{stream, tokenize},
};

fn bench_side(c: &mut Criterion) {
    let side = tokenize(
        r#"side
        {
            "material" "BRICK/BRICKWALL001A"
            "id" "42"
            "uaxis" "[0 1 0 10] 0.125"
            "smoothing_groups" "1"
            "plane" "(0 0 0) (100 0 0) (100 100 0)"
            "lightmapscale" "32"
            "vaxis" "[1 0 0 20] 0.125"
            "rotation" "90"
        }"#,
    );

    c.bench_function("parse side", |b| {
        b.iter_batched(
            || stream(side.clone()),
            |input| {
                Side::parse(input).unwrap();
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(side_benches, bench_side);
criterion_main!(side_benches);
