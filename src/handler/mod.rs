use parking_lot::RwLock;
use std::fmt;
use std::sync::Arc;

use crate::formatter::Formatter;
use crate::level::LogLevel;
use crate::record::Record;

pub mod console;
pub mod file;

/// Trait defining the interface for log handlers
pub trait Handler: fmt::Debug + Send + Sync {
    /// Get the current log level
    fn level(&self) -> LogLevel;

    /// Set the log level
    fn set_level(&mut self, level: LogLevel);

    /// Check if the handler is enabled
    fn enabled(&self) -> bool;

    /// Enable or disable the handler
    fn set_enabled(&mut self, enabled: bool);

    /// Get the formatter
    fn formatter(&self) -> &Formatter;

    /// Set the formatter
    fn set_formatter(&mut self, formatter: Formatter);

    /// Handle a log record
    fn handle(&mut self, record: &Record) -> bool;
}

/// A type alias for a thread-safe handler reference.
pub type HandlerRef = Arc<RwLock<dyn Handler>>;

/// Creates a new handler reference from a handler.
pub fn new_handler_ref<H: Handler + 'static>(handler: H) -> HandlerRef {
    Arc::new(RwLock::new(handler))
}

/// A null handler that does nothing
#[derive(Debug)]
pub struct NullHandler {
    /// The log level
    level: LogLevel,
    /// Whether the handler is enabled
    enabled: bool,
    /// The formatter
    formatter: Formatter,
}

impl NullHandler {
    /// Create a new null handler with the given log level
    pub fn new(level: LogLevel) -> Self {
        Self {
            level,
            enabled: true,
            formatter: Formatter::new(),
        }
    }
}

impl Handler for NullHandler {
    fn level(&self) -> LogLevel {
        self.level
    }

    fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn formatter(&self) -> &Formatter {
        &self.formatter
    }

    fn set_formatter(&mut self, formatter: Formatter) {
        self.formatter = formatter;
    }

    fn handle(&mut self, record: &Record) -> bool {
        self.enabled && record.level() >= self.level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_handler() {
        let mut handler = NullHandler::new(LogLevel::Info);
        assert_eq!(handler.level(), LogLevel::Info);
        assert!(handler.enabled());

        handler.set_level(LogLevel::Warning);
        assert_eq!(handler.level(), LogLevel::Warning);

        handler.set_enabled(false);
        assert!(!handler.enabled());

        let record = Record::new(
            LogLevel::Error,
            "test",
            None::<String>,
            None::<String>,
            None,
        );
        assert!(!handler.handle(&record));

        let formatter = Formatter::new().with_colors(false);
        handler.set_formatter(formatter);
        assert!(!handler.formatter().use_colors);
    }
}
