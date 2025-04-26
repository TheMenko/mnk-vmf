use criterion::{criterion_group, criterion_main, Criterion};
use vmf::{parser::Parser, types::VersionInfo};

fn bench_versioninfo(c: &mut Criterion) {
    let vinfo = r#"versioninfo
                {
                  "editorversion" "400"
                  "editorbuild" "6157"
                  "mapversion" "16"
                  "formatversion" "100"
                  "prefab" "0"
                }"#;

    c.bench_function("parse versioninfo", |b| {
        b.iter(|| {
            VersionInfo::parse(vinfo).unwrap();
        })
    });
}

criterion_group!(versioninfo_benches, bench_versioninfo);
criterion_main!(versioninfo_benches);
