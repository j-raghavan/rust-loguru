use lazy_static::lazy_static;
use parking_lot::RwLock;
use std::sync::Arc;
use std::fmt;

use crate::handler::Handler;
use crate::level::LogLevel;
use crate::record::Record;

/// The main logger implementation that manages log records and handlers.
pub struct Logger {
    /// The minimum log level that will be processed.
    level: LogLevel,
    /// List of registered handlers.
    handlers: Vec<Arc<RwLock<dyn Handler>>>,
}

impl fmt::Debug for Logger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Logger")
            .field("level", &self.level)
            .field("handlers_count", &self.handlers.len())
            .finish()
    }
}

impl Logger {
    /// Creates a new logger with the given minimum log level.
    ///
    /// # Arguments
    ///
    /// * `level` - The minimum log level to process
    pub fn new(level: LogLevel) -> Self {
        Self {
            level,
            handlers: Vec::new(),
        }
    }

    /// Adds a handler to the logger.
    ///
    /// # Arguments
    ///
    /// * `handler` - The handler to add
    ///
    /// # Returns
    ///
    /// The modified `Logger` instance for method chaining.
    pub fn add_handler(mut self, handler: Arc<RwLock<dyn Handler>>) -> Self {
        self.handlers.push(handler);
        self
    }

    /// Logs a record if its level is at or above the logger's minimum level.
    ///
    /// # Arguments
    ///
    /// * `record` - The record to log
    ///
    /// # Returns
    ///
    /// `true` if the record was successfully logged by at least one handler, `false` otherwise.
    pub fn log(&self, record: &Record) -> bool {
        if record.level() < self.level {
            return false;
        }

        let mut any_handled = false;
        for handler in &self.handlers {
            let handler = handler.read();
            if handler.is_enabled() && record.level() >= handler.level() {
                if handler.handle(record) {
                    any_handled = true;
                }
            }
        }
        any_handled
    }

    /// Returns the current minimum log level.
    pub fn level(&self) -> LogLevel {
        self.level
    }

    /// Sets the minimum log level.
    ///
    /// # Arguments
    ///
    /// * `level` - The new minimum log level
    pub fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }
}

lazy_static! {
    /// The global logger instance.
    static ref GLOBAL_LOGGER: RwLock<Logger> = RwLock::new(Logger::new(LogLevel::Info));
}

/// Initializes the global logger with the given configuration.
///
/// # Arguments
///
/// * `logger` - The logger configuration to use
///
/// # Panics
///
/// Panics if the global logger has already been initialized.
pub fn init(logger: Logger) {
    let mut global = GLOBAL_LOGGER.write();
    *global = logger;
}

/// Returns a reference to the global logger.
pub fn global() -> &'static RwLock<Logger> {
    &GLOBAL_LOGGER
}

/// Logs a record using the global logger.
///
/// # Arguments
///
/// * `record` - The record to log
///
/// # Returns
///
/// `true` if the record was successfully logged by at least one handler, `false` otherwise.
pub fn log(record: &Record) -> bool {
    GLOBAL_LOGGER.read().log(record)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::NullHandler;

    #[test]
    fn test_logger_creation() {
        let logger = Logger::new(LogLevel::Debug);
        assert_eq!(logger.level(), LogLevel::Debug);
    }

    #[test]
    fn test_logger_add_handler() {
        let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
        let logger = Logger::new(LogLevel::Debug).add_handler(handler.clone());
        let record = Record::new(
            LogLevel::Info,
            "Test message",
            "test_module",
            "test.rs",
            42,
        );
        assert!(logger.log(&record));
    }

    #[test]
    fn test_logger_log() {
        let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
        let logger = Logger::new(LogLevel::Debug).add_handler(handler.clone());

        let record = Record::new(
            LogLevel::Info,
            "Test message",
            "test_module",
            "test.rs",
            42,
        );

        assert!(logger.log(&record));

        let record = Record::new(
            LogLevel::Trace,
            "Test message",
            "test_module",
            "test.rs",
            42,
        );

        assert!(!logger.log(&record));
    }

    #[test]
    fn test_global_logger() {
        let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
        let logger = Logger::new(LogLevel::Debug).add_handler(handler.clone());
        init(logger);

        let record = Record::new(
            LogLevel::Info,
            "Test message",
            "test_module",
            "test.rs",
            42,
        );

        assert!(log(&record));
    }
} 