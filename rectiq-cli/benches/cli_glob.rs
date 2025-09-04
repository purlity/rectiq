use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rectiq_cli::discover_files;
use rectiq_test_support::{dataset_medium, dataset_small};

fn bench_cli_glob(c: &mut Criterion) {
    let mut group = c.benchmark_group("cli_glob");
    group.sample_size(10);
    group.warm_up_time(std::time::Duration::from_secs(1));

    let small = dataset_small();
    let small_dir = small.path().to_path_buf();
    group.bench_function(BenchmarkId::new("glob", "S"), |b| {
        b.iter(|| {
            black_box(discover_files(black_box(small_dir.as_path())).unwrap());
        });
    });

    let medium = dataset_medium();
    let medium_dir = medium.path().to_path_buf();
    group.bench_function(BenchmarkId::new("glob", "M"), |b| {
        b.iter(|| {
            black_box(discover_files(black_box(medium_dir.as_path())).unwrap());
        });
    });

    #[cfg(feature = "bench-xl")]
    {
        use rectiq_test_support::dataset_large;
        let large = dataset_large();
        let large_dir = large.path().to_path_buf();
        group.bench_function(BenchmarkId::new("glob", "L"), |b| {
            b.iter(|| {
                black_box(discover_files(black_box(large_dir.as_path())).unwrap());
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_cli_glob);
criterion_main!(benches);
