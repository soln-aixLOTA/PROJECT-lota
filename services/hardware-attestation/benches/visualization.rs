use chrono::{DateTime, Utc};
use std::fs;
use std::path::Path;

/// Represents the result of a single benchmark run
#[derive(Debug)]
pub struct BenchmarkResult {
    pub name: String,
    pub timestamp: DateTime<Utc>,
    pub duration_ns: u64,
    pub iterations: u64,
    pub system_metrics: SystemMetricsSnapshot,
}

/// A snapshot of system metrics at a point in time
#[derive(Debug)]
pub struct SystemMetricsSnapshot {
    pub cpu_usage: Vec<f64>,
    pub memory_usage: Vec<f64>,
    pub gpu_usage: Vec<f64>,
}

/// Visualizes benchmark results by generating HTML reports with charts
pub struct BenchmarkVisualizer {
    results: Vec<BenchmarkResult>,
}

impl BenchmarkVisualizer {
    pub fn new() -> Self {
        Self { results: Vec::new() }
    }

    pub fn add_result(&mut self, result: BenchmarkResult) {
        self.results.push(result);
    }

    pub fn generate_report(&self, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut html = String::from(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Benchmark Results</title>
    <script src="https://cdn.plot.ly/plotly-latest.min.js"></script>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .chart { width: 100%; height: 400px; margin: 20px 0; }
    </style>
</head>
<body>
    <h1>Benchmark Results</h1>
"#,
        );

        // Add summary table
        html.push_str("<h2>Summary</h2><table border='1'><tr><th>Benchmark</th><th>Duration (ns)</th><th>Iterations</th></tr>");
        for result in &self.results {
            html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
                result.name, result.duration_ns, result.iterations
            ));
        }
        html.push_str("</table>");

        // Add CPU usage chart
        let cpu_data = self.results.iter().map(|r| {
            let avg_cpu = r.system_metrics.cpu_usage.iter().fold(0.0_f64, |a, &b| a.max(b)) / r.system_metrics.cpu_usage.len() as f64;
            (r.name.clone(), avg_cpu)
        }).collect::<Vec<_>>();

        html.push_str(r#"<div id="cpu_chart" class="chart"></div>"#);
        html.push_str(&format!(
            r#"<script>
                var cpu_data = {{
                    x: {:?},
                    y: {:?},
                    type: 'bar',
                    name: 'CPU Usage'
                }};
                Plotly.newPlot('cpu_chart', [cpu_data], {{
                    title: 'Average CPU Usage per Benchmark',
                    yaxis: {{title: 'CPU Usage (%)'}}
                }});
            </script>"#,
            cpu_data.iter().map(|(name, _)| name).collect::<Vec<_>>(),
            cpu_data.iter().map(|(_, usage)| usage).collect::<Vec<_>>()
        ));

        // Add memory usage chart
        let memory_data = self.results.iter().map(|r| {
            let avg_memory = r.system_metrics.memory_usage.iter().fold(0.0_f64, |a, &b| a.max(b)) / r.system_metrics.memory_usage.len() as f64;
            (r.name.clone(), avg_memory)
        }).collect::<Vec<_>>();

        html.push_str(r#"<div id="memory_chart" class="chart"></div>"#);
        html.push_str(&format!(
            r#"<script>
                var memory_data = {{
                    x: {:?},
                    y: {:?},
                    type: 'bar',
                    name: 'Memory Usage'
                }};
                Plotly.newPlot('memory_chart', [memory_data], {{
                    title: 'Average Memory Usage per Benchmark',
                    yaxis: {{title: 'Memory Usage (MB)'}}
                }});
            </script>"#,
            memory_data.iter().map(|(name, _)| name).collect::<Vec<_>>(),
            memory_data.iter().map(|(_, usage)| usage).collect::<Vec<_>>()
        ));

        html.push_str("</body></html>");
        fs::write(output_path, html)?;
        Ok(())
    }
}
