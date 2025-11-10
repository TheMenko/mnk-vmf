use criterion::{Criterion, criterion_group, criterion_main};
use mnk_vmf::{
    Parser,
    types::Camera,
    util::{stream, tokenize},
};

fn bench_camera(c: &mut Criterion) {
    let input_str = r#"camera
        {
            "id" "42"
            "classname" "point_viewcontrol"
            "origin" "100 200 64"
            "angles" "0 90 0"
            "targetname" "camera_main"
            "spawnflags" "8"
            "fov" "75"
            "speed" "100"
            "acceleration" "500"
            "deceleration" "500"
        }"#;

    // Validate the input parses correctly before benchmarking
    let test_tokens = tokenize(input_str);
    Camera::parse(stream(test_tokens)).expect("Benchmark input should be valid Camera");

    let camera = tokenize(input_str);

    c.bench_function("parse camera", |b| {
        b.iter_batched(
            || stream(camera.clone()),
            |input| {
                Camera::parse(input).unwrap();
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(camera_benches, bench_camera);
criterion_main!(camera_benches);
