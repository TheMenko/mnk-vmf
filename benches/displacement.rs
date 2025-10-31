use criterion::{criterion_group, criterion_main, Criterion};
use vmf::{
    types::DispInfo,
    util::{stream, tokenize},
    Parser,
};

fn bench_dispinfo(c: &mut Criterion) {
    let input_str = r#"dispinfo
        {
            "power" "3"
            "startposition" "[128 256 64]"
            "elevation" "10"
            "subdiv" "1"
            "flags" "0"
            normals
            {
                "row0" "0 0 1 0 0 1 0 0 1 0 0 1 0 0 1"
                "row1" "0 0 1 0 0 1 0 0 1 0 0 1 0 0 1"
                "row2" "0 0 1 0 0 1 0 0 1 0 0 1 0 0 1"
            }
            distances
            {
                "row0" "0 0 0 0 0"
                "row1" "0 0 0 0 0"
                "row2" "0 0 0 0 0"
            }
            offsets
            {
                "row0" "0 0 0 0 0 0 0 0 0 0 0 0 0 0 0"
                "row1" "0 0 0 0 0 0 0 0 0 0 0 0 0 0 0"
            }
            offset_normals
            {
                "row0" "0 0 0 0 0 0 0 0 0 0 0 0 0 0 0"
                "row1" "0 0 0 0 0 0 0 0 0 0 0 0 0 0 0"
            }
            alphas
            {
                "row0" "0 0 0 0 0"
                "row1" "0 0 0 0 0"
            }
            triangle_tags
            {
                "row0" "0 0 0 0 0"
                "row1" "0 0 0 0 0"
            }
            allowed_verts
            {
                "10" "0 1 2 3 4 5 6 7 8 9"
            }
        }"#;

    // Validate the input parses correctly before benchmarking
    let test_tokens = tokenize(input_str);
    DispInfo::parse(stream(test_tokens)).expect("Benchmark input should be valid DispInfo");

    let dispinfo = tokenize(input_str);

    c.bench_function("parse dispinfo", |b| {
        b.iter_batched(
            || stream(dispinfo.clone()),
            |input| {
                DispInfo::parse(input).unwrap();
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(dispinfo_benches, bench_dispinfo);
criterion_main!(dispinfo_benches);
