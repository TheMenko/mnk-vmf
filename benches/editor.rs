use criterion::{Criterion, criterion_group, criterion_main};
use mnk_vmf::{
    Parser,
    types::EditorData,
    util::{stream, tokenize},
};

fn bench_editor(c: &mut Criterion) {
    let input_str = r#"editor
        {
            "color" "0 111 152"
            "visgroupshown" "1"
            "visgroupautoshown" "1"
            "logicalpos" "[0 10000]"
            "comments" "Test comment"
        }"#;

    // Validate the input parses correctly before benchmarking
    let test_tokens = tokenize(input_str);
    EditorData::parse(stream(test_tokens)).expect("Benchmark input should be valid EditorData");

    let editor = tokenize(input_str);

    c.bench_function("parse editor", |b| {
        b.iter_batched(
            || stream(editor.clone()),
            |input| {
                EditorData::parse(input).unwrap();
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(editor_benches, bench_editor);
criterion_main!(editor_benches);
