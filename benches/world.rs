use criterion::{criterion_group, criterion_main, Criterion};
use mnk_vmf::{
    types::World,
    util::{stream, tokenize},
    Parser,
};

fn bench_world(c: &mut Criterion) {
    let input_str = r#"world
        {
            "id" "1"
            "mapversion" "16"
            "classname" "worldspawn"
            "detailmaterial" "detail/detailsprites"
            "detailvbsp" "detail.vbsp"
            "maxpropscreenwidth" "-1"
            "skyname" "sky_day01_01"
            solid
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
            }
            solid
            {
                "id" "30"
                side
                {
                    "id" "30"
                    "plane" "(-384 320 192) (-320 320 192) (-320 -320 192)"
                    "material" "DEV/DEV_MEASUREGENERIC01"
                    "uaxis" "[1 0 0 0] 0.25"
                    "vaxis" "[0 -1 0 0] 0.25"
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
            }
        }"#;

    // Validate the input parses correctly before benchmarking
    let test_tokens = tokenize(input_str);
    World::parse(stream(test_tokens)).expect("Benchmark input should be valid World");

    let world = tokenize(input_str);

    c.bench_function("parse world", |b| {
        b.iter_batched(
            || stream(world.clone()),
            |input| {
                World::parse(input).unwrap();
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(world_benches, bench_world);
criterion_main!(world_benches);
