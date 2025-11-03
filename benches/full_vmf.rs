use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::path::Path;
use vmf::vmf::VMF;

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
                let src = vmf.as_str().expect("Failed to get string");
                b.iter(|| {
                    let tokens = vmf::util::tokenize(black_box(src));
                    black_box(tokens);
                });
            },
        );
    }

    group.finish();
}

fn bench_memory_comparison(c: &mut Criterion) {
    let filename = "Gm_RunDownTown.vmf";
    let path = Path::new(filename);

    if !path.exists() {
        eprintln!("Skipping memory comparison - {} not found", filename);
        return;
    }

    let mut group = c.benchmark_group("memory_comparison");

    // Benchmark: mmap-based approach (current)
    group.bench_function("mmap_approach", |b| {
        b.iter(|| {
            let vmf = VMF::open(path).expect("Failed to open VMF");
            let data = vmf.parse().expect("Failed to parse VMF");
            black_box(data);
        });
    });

    // Benchmark: traditional read_to_string approach
    group.bench_function("read_to_string_approach", |b| {
        b.iter(|| {
            let contents = std::fs::read_to_string(path).expect("Failed to read file");
            let tokens = vmf::util::tokenize(black_box(&contents));
            black_box(tokens);
            // Note: Can't parse since it needs the VMF struct
        });
    });

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

criterion_group!(
    benches,
    bench_full_vmf_parsing,
    bench_memory_comparison,
    bench_incremental_access
);
criterion_main!(benches);
