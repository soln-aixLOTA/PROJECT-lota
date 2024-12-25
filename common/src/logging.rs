use tracing::{Level, Subscriber};
use tracing_subscriber::{FmtSubscriber, EnvFilter};
use uuid::Uuid;

pub struct LoggingConfig {
    pub service_name: String,
    pub environment: String,
    pub log_level: Level,
    pub json_format: bool,
}

impl LoggingConfig {
    pub fn init(&self) {
        let subscriber = FmtSubscriber::builder()
            .with_env_filter(EnvFilter::from_default_env())
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .with_level(true)
            .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
            .json(self.json_format)
            .with_current_span(true)
            .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
            .init();
    }
}

pub fn request_id() -> String {
    Uuid::new_v4().to_string()
}
