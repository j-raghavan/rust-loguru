use lazy_static::lazy_static;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::handler::Handler;
use crate::level::LogLevel;
use crate::record::Record;
use crate::AsyncLoggerBuilder;
use crate::AsyncLoggerHandle;

/// Debug print macro that only prints when the debug_logging feature is enabled
#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        #[cfg(feature = "debug_logging")]
        println!($($arg)*);
    };
}

/// A logger that can handle log records
#[derive(Debug)]
pub struct Logger {
    /// The log level
    level: LogLevel,
    /// The handlers
    handlers: Vec<Arc<RwLock<dyn Handler>>>,
    /// Whether async logging is enabled
    async_mode: bool,
    /// The async logger handle
    async_handle: Option<AsyncLoggerHandle>,
    /// Whether the logger is active
    active: Arc<AtomicBool>,
}

impl Logger {
    /// Create a new logger with the given log level
    pub fn new(level: LogLevel) -> Self {
        Self {
            level,
            handlers: Vec::new(),
            async_mode: false,
            async_handle: None,
            active: Arc::new(AtomicBool::new(true)),
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

    /// Enable or disable async logging
    pub fn set_async(&mut self, enable: bool, queue_size: Option<usize>) -> &mut Self {
        // Don't make any changes if already in the requested state
        if self.async_mode == enable {
            return self;
        }

        if enable {
            // Create a new async logger if needed
            let builder = AsyncLoggerBuilder::new()
                .with_queue_size(queue_size.unwrap_or(10000))
                .with_handlers(self.handlers.clone())
                .with_level(self.level);

            self.async_handle = Some(builder.build());
            self.async_mode = true;
        } else {
            // Shut down the async logger
            if let Some(handle) = self.async_handle.take() {
                handle.shutdown();
            }
            self.async_mode = false;
        }

        self
    }

    /// Set the number of worker threads for async logging
    pub fn set_worker_threads(&mut self, count: usize) -> &mut Self {
        if self.async_mode && self.async_handle.is_some() {
            // Recreate the async logger with the new worker count
            self.set_async(false, None);
            let builder = AsyncLoggerBuilder::new()
                .with_queue_size(10000)
                .with_handlers(self.handlers.clone())
                .with_level(self.level)
                .with_workers(count);

            self.async_handle = Some(builder.build());
            self.async_mode = true;
        }
        self
    }

    /// Log a record
    pub fn log(&self, record: &Record) -> bool {
        println!(
            "Logger::log - record level: {:?}, logger level: {:?}",
            record.level(),
            self.level
        );
        if record.level() < self.level || !self.active.load(Ordering::Relaxed) {
            println!("Logger::log - record level < logger level or logger not active");
            return false;
        }

        // If async logging is enabled, dispatch to the async logger
        if self.async_mode {
            if let Some(handle) = &self.async_handle {
                return handle.log(record.clone());
            }
        }

        // Otherwise, log synchronously
        self.log_sync(record)
    }

    /// Log a record synchronously
    fn log_sync(&self, record: &Record) -> bool {
        let mut any_handled = false;
        debug_println!(
            "log_sync: record level = {:?}, logger level = {:?}",
            record.level(),
            self.level
        );
        for handler in &self.handlers {
            let mut guard = handler.write();
            debug_println!(
                "log_sync: handler enabled = {}, handler level = {:?}",
                guard.enabled(),
                guard.level()
            );
            if guard.enabled() && record.level() >= guard.level() {
                debug_println!("log_sync: calling handle on handler");
                if guard.handle(record) {
                    debug_println!("log_sync: handler returned true");
                    any_handled = true;
                } else {
                    debug_println!("log_sync: handler returned false");
                }
            } else {
                debug_println!("log_sync: skipping handler due to level/enabled check");
            }
        }
        debug_println!("log_sync: returning {}", any_handled);
        any_handled
    }

    /// Log a message at the given level with fluent API
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

    /// Enable or disable the logger
    pub fn set_enabled(&self, enabled: bool) -> &Self {
        self.active.store(enabled, Ordering::Relaxed);
        self
    }

    /// Check if the logger is enabled
    pub fn is_enabled(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    /// Get all registered handlers
    pub fn handlers(&self) -> &[Arc<RwLock<dyn Handler>>] {
        &self.handlers
    }

    /// Check if async logging is enabled
    pub fn is_async(&self) -> bool {
        self.async_mode
    }
}

impl Clone for Logger {
    fn clone(&self) -> Self {
        Self {
            level: self.level,
            handlers: self.handlers.clone(),
            async_mode: self.async_mode,
            async_handle: self.async_handle.clone(),
            active: Arc::new(AtomicBool::new(self.active.load(Ordering::Relaxed))),
        }
    }
}

lazy_static! {
    /// The global logger instance.
    static ref GLOBAL_LOGGER: RwLock<Logger> = RwLock::new(Logger::new(LogLevel::Info));
}

/// Initialize the global logger
pub fn init(logger: Logger) -> Logger {
    let mut global = GLOBAL_LOGGER.write();
    *global = logger.clone();
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

    #[test]
    fn test_logger_enabled() {
        let logger = Logger::new(LogLevel::Info);
        assert!(logger.is_enabled());

        logger.set_enabled(false);
        assert!(!logger.is_enabled());

        logger.set_enabled(true);
        assert!(logger.is_enabled());
    }

    #[test]
    fn test_async_logging() {
        let mut logger = Logger::new(LogLevel::Info);
        let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
        logger.add_handler(handler);

        // Test sync logging first
        assert!(logger.info("Sync message"));
        assert!(!logger.is_async());

        // Enable async logging
        logger.set_async(true, Some(1000));
        assert!(logger.is_async());

        // Test async logging
        assert!(logger.info("Async message"));

        // Test changing worker threads
        logger.set_worker_threads(2);
        assert!(logger.info("Message with 2 workers"));

        // Disable async logging
        logger.set_async(false, None);
        assert!(!logger.is_async());

        // Test sync logging again
        assert!(logger.info("Sync message again"));
    }

    #[test]
    fn test_compile_time_filtering() {
        let logger = Logger::new(LogLevel::Info);

        // This would be optimized out at compile time with feature flags
        if !crate::compile_time_level_enabled!(LogLevel::Debug) {
            assert!(!logger.debug("Debug message"));
        }

        // This would pass compile-time check and then be checked at runtime
        if crate::compile_time_level_enabled!(LogLevel::Info) {
            // Runtime check happens inside logger.log
            let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
            let mut logger = Logger::new(LogLevel::Info);
            logger.add_handler(handler);
            assert!(logger.info("Info message"));
        }
    }
}
