use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rectiq_cli::run_with_args;
use rectiq_test_support::{dataset_medium, dataset_small};

fn bench_cli_e2e(c: &mut Criterion) {
    unsafe {
        std::env::set_var("RECTIQ_RAYON_THREADS", num_cpus::get_physical().to_string());
    }
    let mut group = c.benchmark_group("cli_e2e");
    group.sample_size(10);
    group.warm_up_time(std::time::Duration::from_secs(1));

    let small = dataset_small();
    let small_dir = small.path().to_string_lossy().to_string();
    group.bench_function(BenchmarkId::new("scan", "S"), |b| {
        b.iter(|| {
            run_with_args(["rectiq", "scan", black_box(small_dir.as_str())]).unwrap();
        });
    });

    let medium = dataset_medium();
    let medium_dir = medium.path().to_string_lossy().to_string();
    group.bench_function(BenchmarkId::new("scan", "M"), |b| {
        b.iter(|| {
            run_with_args(["rectiq", "scan", black_box(medium_dir.as_str())]).unwrap();
        });
    });

    #[cfg(feature = "bench-xl")]
    {
        use rectiq_test_support::dataset_large;
        let large = dataset_large();
        let large_dir = large.path().to_string_lossy().to_string();
        group.bench_function(BenchmarkId::new("scan", "L"), |b| {
            b.iter(|| {
                run_with_args(["rectiq", "scan", black_box(large_dir.as_str())]).unwrap();
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_cli_e2e);
criterion_main!(benches);
