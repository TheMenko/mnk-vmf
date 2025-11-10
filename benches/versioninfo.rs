use criterion::{Criterion, criterion_group, criterion_main};
use mnk_vmf::{
    Parser,
    types::VersionInfo,
    util::{stream, tokenize},
};

fn bench_versioninfo(c: &mut Criterion) {
    let vinfo = tokenize(
        r#"versioninfo
                {
                  "editorversion" "400"
                  "editorbuild" "6157"
                  "mapversion" "16"
                  "formatversion" "100"
                  "prefab" "0"
                }"#,
    );

    c.bench_function("parse versioninfo", |b| {
        b.iter_batched(
            || stream(vinfo.clone()),
            |input| {
                VersionInfo::parse(input).unwrap();
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(versioninfo_benches, bench_versioninfo);
criterion_main!(versioninfo_benches);
