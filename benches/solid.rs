use criterion::{criterion_group, criterion_main, Criterion};
use mnk_vmf::{
    types::Solid,
    util::{stream, tokenize},
    Parser,
};

fn bench_solid(c: &mut Criterion) {
    let input_str = r#"solid
        {
            "id" "9"
            side
            {
                "id" "1"
                "plane" "(-320 -320 0) (-320 320 0) (320 320 0)"
                "material" "DEV/DEV_MEASUREGENERIC01B"
                "uaxis" "[1 0 0 0] 0.25"
                "vaxis" "[0 -1 0 0] 0.25"
                "rotation" "0"
                "lightmapscale" "16"
                "smoothing_groups" "0"
            }
            side
            {
                "id" "2"
                "plane" "(-320 320 -64) (-320 -320 -64) (320 -320 -64)"
                "material" "DEV/DEV_MEASUREGENERIC01B"
                "uaxis" "[1 0 0 0] 0.25"
                "vaxis" "[0 -1 0 0] 0.25"
                "rotation" "0"
                "lightmapscale" "16"
                "smoothing_groups" "0"
            }
            side
            {
                "id" "3"
                "plane" "(-320 -320 -64) (-320 320 -64) (-320 320 0)"
                "material" "DEV/DEV_MEASUREGENERIC01B"
                "uaxis" "[0 1 0 0] 0.25"
                "vaxis" "[0 0 -1 0] 0.25"
                "rotation" "0"
                "lightmapscale" "16"
                "smoothing_groups" "0"
            }
            editor
            {
                "color" "0 111 152"
                "visgroupshown" "1"
                "visgroupautoshown" "1"
            }
        }"#;

    // Validate the input parses correctly before benchmarking
    let test_tokens = tokenize(input_str);
    Solid::parse(stream(test_tokens)).expect("Benchmark input should be valid Solid");

    let solid = tokenize(input_str);

    c.bench_function("parse solid", |b| {
        b.iter_batched(
            || stream(solid.clone()),
            |input| {
                Solid::parse(input).unwrap();
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(solid_benches, bench_solid);
criterion_main!(solid_benches);
