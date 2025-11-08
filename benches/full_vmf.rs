use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use mnk_vmf::vmf::VMF;
use std::path::Path;

fn bench_full_vmf_parsing(c: &mut Criterion) {
    let test_files = [
        ("test.vmf", "Small test file"),
        ("Gm_RunDownTown.vmf", "Real 15MB map"),
    ];

    let mut group = c.benchmark_group("full_vmf_parsing");

    for (filename, description) in test_files.iter() {
        let path = Path::new(filename);

        // Skip if file doesn't exist
        if !path.exists() {
            eprintln!("Skipping {} - file not found", filename);
            continue;
        }

        // Get file size for throughput measurement
        let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

        group.throughput(Throughput::Bytes(file_size));

        // Benchmark: Open + Parse (complete workflow)
        group.bench_with_input(
            BenchmarkId::new("open_and_parse", description),
            filename,
            |b, &filename| {
                b.iter(|| {
                    let vmf = VMF::open(Path::new(filename)).expect("Failed to open VMF");
                    let data = vmf.parse().expect("Failed to parse VMF");
                    black_box(data);
                });
            },
        );

        // Benchmark: Just parsing (VMF already opened)
        group.bench_with_input(
            BenchmarkId::new("parse_only", description),
            filename,
            |b, &filename| {
                let vmf = VMF::open(Path::new(filename)).expect("Failed to open VMF");
                b.iter(|| {
                    let data = vmf.parse().expect("Failed to parse VMF");
                    black_box(data);
                });
            },
        );

        // Benchmark: Just tokenization
        group.bench_with_input(
            BenchmarkId::new("tokenize_only", description),
            filename,
            |b, &filename| {
                let vmf = VMF::open(Path::new(filename)).expect("Failed to open VMF");
                let src = vmf.as_str();
                b.iter(|| {
                    let tokens = mnk_vmf::util::tokenize(black_box(src));
                    black_box(tokens);
                });
            },
        );
    }

    group.finish();
}

fn bench_incremental_access(c: &mut Criterion) {
    let filename = "Gm_RunDownTown.vmf";
    let path = Path::new(filename);

    if !path.exists() {
        eprintln!("Skipping incremental access - {} not found", filename);
        return;
    }

    let mut group = c.benchmark_group("incremental_access");

    // Benchmark: Parse multiple times (shows mmap caching benefit)
    group.bench_function("multiple_parses", |b| {
        let vmf = VMF::open(path).expect("Failed to open VMF");
        b.iter(|| {
            for _ in 0..3 {
                let data = vmf.parse().expect("Failed to parse VMF");
                black_box(data);
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_full_vmf_parsing, bench_incremental_access);
criterion_main!(benches);
