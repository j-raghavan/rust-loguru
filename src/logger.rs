use lazy_static::lazy_static;
use parking_lot::RwLock;
use std::sync::Arc;

use crate::handler::Handler;
use crate::level::LogLevel;
use crate::record::Record;

/// A logger that can handle log records
#[derive(Debug)]
pub struct Logger {
    /// The log level
    level: LogLevel,
    /// The handlers
    handlers: Vec<Arc<RwLock<dyn Handler>>>,
}

impl Logger {
    /// Create a new logger with the given log level
    pub fn new(level: LogLevel) -> Self {
        Self {
            level,
            handlers: Vec::new(),
        }
    }

    /// Get the log level
    pub fn level(&self) -> LogLevel {
        self.level
    }

    /// Set the log level
    pub fn set_level(&mut self, level: LogLevel) -> &mut Self {
        self.level = level;
        self
    }

    /// Add a handler to the logger
    pub fn add_handler(&mut self, handler: Arc<RwLock<dyn Handler>>) -> &mut Self {
        self.handlers.push(handler);
        self
    }

    /// Remove a handler from the logger
    pub fn remove_handler(&mut self, handler: Arc<RwLock<dyn Handler>>) -> &mut Self {
        self.handlers.retain(|h| !Arc::ptr_eq(h, &handler));
        self
    }

    /// Log a record
    pub fn log(&self, record: &Record) -> bool {
        if record.level() < self.level {
            return false;
        }

        let mut any_handled = false;
        for handler in &self.handlers {
            let mut guard = handler.write();
            if guard.enabled() && record.level() >= guard.level() && guard.handle(record) {
                any_handled = true;
            }
        }
        any_handled
    }

    /// Log a message at the given level
    pub fn log_message(&self, level: LogLevel, message: impl Into<String>) -> bool {
        let record = Record::new(level, message, None::<String>, None::<String>, None);
        self.log(&record)
    }

    /// Log a debug message
    pub fn debug(&self, message: impl Into<String>) -> bool {
        self.log_message(LogLevel::Debug, message)
    }

    /// Log an info message
    pub fn info(&self, message: impl Into<String>) -> bool {
        self.log_message(LogLevel::Info, message)
    }

    /// Log a warning message
    pub fn warn(&self, message: impl Into<String>) -> bool {
        self.log_message(LogLevel::Warning, message)
    }

    /// Log an error message
    pub fn error(&self, message: impl Into<String>) -> bool {
        self.log_message(LogLevel::Error, message)
    }
}

lazy_static! {
    /// The global logger instance.
    static ref GLOBAL_LOGGER: RwLock<Logger> = RwLock::new(Logger::new(LogLevel::Info));
}

/// Initialize the global logger
pub fn init(logger: Logger) -> Logger {
    logger
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
        let logger = Logger::new(LogLevel::Info);
        assert_eq!(logger.level(), LogLevel::Info);
        assert!(logger.handlers.is_empty());
    }

    #[test]
    fn test_logger_level() {
        let mut logger = Logger::new(LogLevel::Info);
        logger.set_level(LogLevel::Debug);
        assert_eq!(logger.level(), LogLevel::Debug);
    }

    #[test]
    fn test_logger_handler() {
        let mut logger = Logger::new(LogLevel::Info);
        let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
        logger.add_handler(handler.clone());
        assert_eq!(logger.handlers.len(), 1);
        logger.remove_handler(handler);
        assert!(logger.handlers.is_empty());
    }

    #[test]
    fn test_logger_log() {
        let mut logger = Logger::new(LogLevel::Info);
        let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
        logger.add_handler(handler);

        // Test logging at different levels
        assert!(!logger.debug("Debug message")); // Should not log (below Info)
        assert!(logger.info("Info message")); // Should log
        assert!(logger.warn("Warning message")); // Should log
        assert!(logger.error("Error message")); // Should log
    }

    #[test]
    fn test_logger_level_filtering() {
        let mut logger = Logger::new(LogLevel::Warning);
        let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
        logger.add_handler(handler);

        // Test level filtering
        assert!(!logger.debug("Debug message")); // Should not log
        assert!(!logger.info("Info message")); // Should not log
        assert!(logger.warn("Warning message")); // Should log
        assert!(logger.error("Error message")); // Should log
    }

    #[test]
    fn test_logger_handler_filtering() {
        let mut logger = Logger::new(LogLevel::Info);
        let mut handler = NullHandler::new(LogLevel::Info);
        handler.set_level(LogLevel::Warning);
        let handler = Arc::new(RwLock::new(handler));
        logger.add_handler(handler);

        // Test handler level filtering
        assert!(!logger.debug("Debug message")); // Should not log
        assert!(!logger.info("Info message")); // Should not log
        assert!(logger.warn("Warning message")); // Should log
        assert!(logger.error("Error message")); // Should log
    }

    #[test]
    fn test_logger_disabled_handler() {
        let mut logger = Logger::new(LogLevel::Info);
        let mut handler = NullHandler::new(LogLevel::Info);
        handler.set_enabled(false);
        let handler = Arc::new(RwLock::new(handler));
        logger.add_handler(handler);

        // Test disabled handler
        assert!(!logger.info("Info message")); // Should not log
    }

    #[test]
    fn test_logger_init() {
        let logger = Logger::new(LogLevel::Info);
        let logger = init(logger);
        assert_eq!(logger.level(), LogLevel::Info);
    }
}
