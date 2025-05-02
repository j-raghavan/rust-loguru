use parking_lot::RwLock;
use std::fmt;
use std::io;
use std::sync::Arc;

use crate::formatters::Formatter;
use crate::level::LogLevel;
use crate::record::Record;

/// A type alias for a handler filter function
pub type HandlerFilter = Arc<dyn Fn(&Record) -> bool + Send + Sync>;

/// Error type for handler operations
#[derive(Debug)]
pub enum HandlerError {
    /// An I/O error occurred
    IoError(io::Error),
    /// The handler is not initialized
    NotInitialized,
    /// A custom error message
    Custom(String),
}

impl std::fmt::Display for HandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandlerError::IoError(e) => write!(f, "I/O error: {}", e),
            HandlerError::NotInitialized => write!(f, "Handler not initialized"),
            HandlerError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for HandlerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HandlerError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

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

    /// Set a filter closure for this handler (optional, default: no filter)
    fn set_filter(&mut self, filter: Option<HandlerFilter>);

    /// Get the filter closure for this handler (optional)
    fn filter(&self) -> Option<&HandlerFilter>;

    /// Handle a log record
    fn handle(&self, record: &Record) -> Result<(), HandlerError>;

    /// Handle a batch of log records (optimized implementation)
    fn handle_batch(&self, records: &[Record]) -> Result<(), HandlerError> {
        for record in records {
            self.handle(record)?;
        }
        Ok(())
    }

    /// Lifecycle: initialize the handler
    fn init(&mut self) -> Result<(), HandlerError> {
        Ok(())
    }

    /// Lifecycle: flush the handler
    fn flush(&self) -> Result<(), HandlerError> {
        Ok(())
    }

    /// Lifecycle: shutdown the handler
    fn shutdown(&mut self) -> Result<(), HandlerError> {
        Ok(())
    }
}

/// A handler that does nothing
pub struct NullHandler {
    /// The log level
    level: LogLevel,
    /// Whether the handler is enabled
    enabled: bool,
    /// The formatter to use
    formatter: Formatter,
    /// Optional filter closure
    filter: Option<HandlerFilter>,
}

impl Default for NullHandler {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            enabled: true,
            formatter: Formatter::text(),
            filter: None,
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

    fn set_filter(&mut self, filter: Option<HandlerFilter>) {
        self.filter = filter;
    }

    fn filter(&self) -> Option<&HandlerFilter> {
        self.filter.as_ref()
    }

    fn handle(&self, _record: &Record) -> Result<(), HandlerError> {
        Ok(())
    }
}

impl NullHandler {
    pub fn new(level: LogLevel) -> Self {
        Self {
            level,
            enabled: true,
            formatter: Formatter::text(),
            filter: None,
        }
    }
}

impl fmt::Debug for NullHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NullHandler")
            .field("level", &self.level)
            .field("enabled", &self.enabled)
            .field("formatter", &self.formatter)
            .finish()
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
pub struct BaseHandler {
    /// The log level
    level: LogLevel,
    /// Whether the handler is enabled
    enabled: bool,
    /// The formatter
    formatter: Formatter,
    /// Optional filter closure
    filter: Option<HandlerFilter>,
}

impl BaseHandler {
    /// Create a new handler
    pub fn new(level: LogLevel) -> Self {
        Self {
            level,
            enabled: true,
            formatter: Formatter::text(),
            filter: None,
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

    fn set_filter(&mut self, filter: Option<HandlerFilter>) {
        self.filter = filter;
    }

    fn filter(&self) -> Option<&HandlerFilter> {
        self.filter.as_ref()
    }

    fn handle(&self, record: &Record) -> Result<(), HandlerError> {
        if !self.is_enabled() {
            return Ok(());
        }

        if let Some(filter) = &self.filter {
            if !filter(record) {
                return Ok(());
            }
        }

        Ok(())
    }
}

impl fmt::Debug for BaseHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BaseHandler")
            .field("level", &self.level)
            .field("enabled", &self.enabled)
            .field("formatter", &self.formatter)
            .finish()
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
