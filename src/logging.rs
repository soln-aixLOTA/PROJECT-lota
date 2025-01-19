use log::{Level, LevelFilter, Metadata, Record};
use serde_json::json;
use std::io::{self, Write};

pub struct JsonLogger;

impl log::Log for JsonLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let log_entry = json!({
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "level": record.level().to_string(),
                "target": record.target(),
                "message": record.args(),
                "module_path": record.module_path(),
                "file": record.file(),
                "line": record.line(),
            });

            writeln!(io::stdout(), "{}", log_entry).unwrap();
        }
    }

    fn flush(&self) {}
}

pub fn init_logger() {
    log::set_boxed_logger(Box::new(JsonLogger))
        .map(|()| log::set_max_level(LevelFilter::Info))
        .unwrap();
}
