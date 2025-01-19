use criterion::{criterion_group, criterion_main, Criterion};
use hardware_attestation::NvmlWrapper;
use tokio::runtime::Runtime;
use futures::future::join_all;

mod bench_utils;
use bench_utils::configure_group;

mod system_monitor;
use system_monitor::MonitoredBenchmark;

fn bench_memory_info(c: &mut Criterion) {
    let mut group = configure_group(c, "gpu_memory_info");
    let rt = Runtime::new().unwrap();

    group.bench_function("sequential", |b| {
        b.iter_custom(|iters| {
            rt.block_on(async {
                let nvml = NvmlWrapper::new().expect("Failed to initialize NVML");
                let benchmark = MonitoredBenchmark::new();
                benchmark.run(iters, || async {
                    let _ = nvml.get_device_memory_info(0);
                }).await
            })
        });
    });

    group.bench_function("concurrent_10", |b| {
        b.iter_custom(|iters| {
            rt.block_on(async {
                let nvml = NvmlWrapper::new().expect("Failed to initialize NVML");
                let benchmark = MonitoredBenchmark::new();
                let start = std::time::Instant::now();
                let mut tasks = Vec::new();
                for _ in 0..10 {
                    tasks.push(benchmark.run(iters / 10, || async {
                        let _ = nvml.get_device_memory_info(0);
                    }));
                }
                join_all(tasks).await;
                start.elapsed()
            })
        });
    });

    group.finish();
}

fn bench_device_count(c: &mut Criterion) {
    let mut group = configure_group(c, "gpu_device_count");
    let rt = Runtime::new().unwrap();

    group.bench_function("sequential", |b| {
        b.iter_custom(|iters| {
            rt.block_on(async {
                let nvml = NvmlWrapper::new().expect("Failed to initialize NVML");
                let benchmark = MonitoredBenchmark::new();
                benchmark.run(iters, || async {
                    let _ = nvml.get_device_count();
                }).await
            })
        });
    });

    group.finish();
}

criterion_group!(benches, bench_memory_info, bench_device_count);
criterion_main!(benches);
