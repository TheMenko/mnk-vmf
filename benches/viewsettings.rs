use criterion::{Criterion, criterion_group, criterion_main};
use mnk_vmf::{
    Parser,
    types::ViewSettings,
    util::{stream, tokenize},
};

fn bench_versioninfo(c: &mut Criterion) {
    let vinfo = tokenize(
        r#"viewsettings
        {
            "bSnapToGrid" "1"
            "bShowGrid" "1"
            "bShowLogicalGrid" "0"
            "nGridSpacing" "64"
            "bShow3DGrid" "1"
            "bHideObjects" "0"
            "bHideWalls" "1"
            "bHideStripes" "0"
            "bHideNeighbors" "1"
            "bHideDetail" "0"
            "bShowBrushes" "1"
            "bShowEntities" "0"
            "bShowLightRadius" "1"
            "bShowLightingPreview" "0"
            "bShowWireframe" "1"
        }"#,
    );

    c.bench_function("parse viewsettings", |b| {
        b.iter_batched(
            || stream(vinfo.clone()),
            |input| {
                ViewSettings::parse(input).unwrap();
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(versioninfo_benches, bench_versioninfo);
criterion_main!(versioninfo_benches);
