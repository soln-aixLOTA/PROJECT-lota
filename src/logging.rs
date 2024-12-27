use serde::{Deserialize, Serialize};
use tracing::{info, Level};
use tracing_subscriber::{fmt::format::FmtSpan, FmtSubscriber};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub file_logging: bool,
    pub file_path: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        LoggingConfig {
            level: "info".to_string(),
            format: "json".to_string(),
            file_logging: false,
            file_path: None,
        }
    }
}

pub fn init_logging(config: &LoggingConfig) {
    let level = match config.level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(true)
        .with_env_filter("info")
        .compact();

    if config.format == "json" {
        subscriber.json().init();
    } else {
        subscriber.init();
    }

    info!("Logging initialized with level: {}", config.level);
}
