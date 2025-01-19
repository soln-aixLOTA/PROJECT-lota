use criterion::{Criterion, BenchmarkGroup, measurement::WallTime};
use std::time::Duration;

pub mod system_monitor;
pub mod visualization;

pub fn configure_group<'a>(c: &'a mut Criterion, name: &str) -> BenchmarkGroup<'a, WallTime> {
    let mut group = c.benchmark_group(name);
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);
    group.warm_up_time(Duration::from_secs(1));
    group
}

/// Helper function to format benchmark names
pub fn format_bench_name(operation: &str, variant: &str) -> String {
    format!("{}_{}", operation, variant)
}

/// Standard measurement configuration
pub const MEASUREMENT_CONFIG: Duration = Duration::from_secs(10);
pub const SAMPLE_SIZE: usize = 100;
pub const WARM_UP_TIME: Duration = Duration::from_secs(2);
