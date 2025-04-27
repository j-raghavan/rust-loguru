use parking_lot::RwLock;
use std::fmt;
use std::sync::Arc;

use crate::formatters::Formatter;
use crate::level::LogLevel;
use crate::record::Record;

/// A trait for handlers that handle log records
pub trait Handler: Send + Sync + fmt::Debug {
    /// Get the log level
    fn level(&self) -> LogLevel;

    /// Set the log level
    fn set_level(&mut self, level: LogLevel);

    /// Check if the handler is enabled
    fn is_enabled(&self) -> bool;

    /// Set whether the handler is enabled
    fn set_enabled(&mut self, enabled: bool);

    /// Get the formatter
    fn formatter(&self) -> &Formatter;

    /// Set the formatter
    fn set_formatter(&mut self, formatter: Formatter);

    /// Handle a log record
    fn handle(&self, record: &Record) -> Result<(), String>;
}

/// A handler that does nothing
#[derive(Debug, Clone)]
pub struct NullHandler {
    /// The log level
    level: LogLevel,
    /// Whether the handler is enabled
    enabled: bool,
    /// The formatter to use
    formatter: Formatter,
}

impl Default for NullHandler {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            enabled: true,
            formatter: Formatter::text(),
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

    fn is_enabled(&self) -> bool {
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

    fn handle(&self, _record: &Record) -> Result<(), String> {
        Ok(())
    }
}

impl NullHandler {
    pub fn new(level: LogLevel) -> Self {
        Self {
            level,
            enabled: true,
            formatter: Formatter::text(),
        }
    }
}

pub mod console;
pub mod file;
pub mod network;

/// A type alias for a thread-safe handler reference.
pub type HandlerRef = Arc<RwLock<dyn Handler>>;

/// Creates a new handler reference from a handler.
pub fn new_handler_ref<H: Handler + 'static>(handler: H) -> HandlerRef {
    Arc::new(RwLock::new(handler))
}

/// Base handler implementation
#[derive(Debug)]
pub struct BaseHandler {
    /// The log level
    level: LogLevel,
    /// Whether the handler is enabled
    enabled: bool,
    /// The formatter
    formatter: Formatter,
}

impl BaseHandler {
    /// Create a new handler
    pub fn new(level: LogLevel) -> Self {
        Self {
            level,
            enabled: true,
            formatter: Formatter::text(),
        }
    }
}

impl Handler for BaseHandler {
    fn level(&self) -> LogLevel {
        self.level
    }

    fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }

    fn is_enabled(&self) -> bool {
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

    fn handle(&self, record: &Record) -> Result<(), String> {
        if !self.enabled || record.level() < self.level {
            return Ok(());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_handler() {
        let mut handler = BaseHandler::new(LogLevel::Info);
        assert_eq!(handler.level(), LogLevel::Info);
        assert!(handler.is_enabled());

        // Test disabled handler
        handler.set_enabled(false);
        assert!(!handler.is_enabled());
        let record = Record::new(LogLevel::Info, "test", None::<String>, None::<String>, None);
        assert!(handler.handle(&record).is_ok());

        // Test level filtering
        handler.set_enabled(true);
        let debug_record = Record::new(
            LogLevel::Debug,
            "test",
            None::<String>,
            None::<String>,
            None,
        );
        assert!(handler.handle(&debug_record).is_ok()); // Should succeed but not log (Debug < Info)

        let info_record = Record::new(LogLevel::Info, "test", None::<String>, None::<String>, None);
        assert!(handler.handle(&info_record).is_ok()); // Should succeed and log

        // Test formatter
        let formatter = Formatter::text();
        handler.set_formatter(formatter);
        assert!(handler.formatter().format(&info_record).contains("test"));
    }
}
