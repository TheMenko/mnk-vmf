use criterion::{criterion_group, criterion_main, Criterion};
use vmf::{
    types::VisGroups,
    util::{stream, tokenize},
    Parser,
};

fn bench_visgroups(c: &mut Criterion) {
    let input = r#"
        visgroups
        {
            visgroup
            {
                "name" "Group A"
                "visgroupid" "1"
                "color" "255 0 0"
            }
            visgroup
            {
                "name" "Group B"
                "visgroupid" "2"
                "color" "0 255 0"
                visgroup
                {
                    "name" "Subgroup B1"
                    "visgroupid" "3"
                    "color" "0 0 255"
                }
            }
        }
    "#;

    let tokens = tokenize(input);

    c.bench_function("parse visgroups", |b| {
        b.iter_batched(
            || stream(tokens.clone()),
            |input| {
                VisGroups::parse(input).unwrap();
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(visgroups_benches, bench_visgroups);
criterion_main!(visgroups_benches);
