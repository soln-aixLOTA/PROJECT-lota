use crate::config::LoggingConfig;
use tracing::{info, Level};
use tracing_subscriber::fmt;

pub fn init_logging(config: &LoggingConfig) {
    let level = match config.level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let format = config.format.to_lowercase();
    let is_json = format == "json";

    let builder = fmt::Subscriber::builder()
        .with_max_level(level)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .with_thread_names(true)
        .with_ansi(true);

    let subscriber = if is_json {
        builder.json().try_init()
    } else {
        builder.try_init()
    };

    match subscriber {
        Ok(_) => info!(
            "Logging initialized with level: {}, format: {}",
            config.level, config.format
        ),
        Err(e) => eprintln!("Failed to initialize logging: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_logging() {
        let config = LoggingConfig {
            level: "info".to_string(),
            format: "text".to_string(),
            output: "stdout".to_string(),
        };
        init_logging(&config);
    }
}
