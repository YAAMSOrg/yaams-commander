//! Background activity log on stdout, visible when the app is started from a
//! terminal. Every line carries a `DEBUG:` prefix.

use std::io::Write;

use log::{LevelFilter, Log, Metadata, Record};

struct StdoutLogger;

impl Log for StdoutLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // Only our own crates; dependencies (rustls, reqwest, ...) are too noisy.
        metadata.target().starts_with("yaams")
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            // Ignore write errors (closed pipe, no console on Windows).
            let _ = writeln!(std::io::stdout().lock(), "DEBUG: {}", record.args());
        }
    }

    fn flush(&self) {
        let _ = std::io::stdout().flush();
    }
}

/// Installs the stdout logger. Calling it more than once has no effect.
pub fn init() {
    if log::set_logger(&StdoutLogger).is_ok() {
        log::set_max_level(LevelFilter::Debug);
    }
}
