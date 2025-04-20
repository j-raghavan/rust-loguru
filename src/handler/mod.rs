use crate::level::LogLevel;
use crate::record::Record;

/// Trait defining the interface for log handlers.
/// Handlers are responsible for processing and outputting log records.
pub trait Handler: Send + Sync {
    /// Processes a log record.
    ///
    /// # Arguments
    ///
    /// * `record` - The log record to process
    ///
    /// # Returns
    ///
    /// `true` if the record was successfully processed, `false` otherwise.
    fn handle(&self, record: &Record) -> bool;

    /// Returns the minimum log level that this handler will process.
    /// Records with levels below this threshold will be ignored.
    fn level(&self) -> LogLevel;

    /// Sets the minimum log level for this handler.
    ///
    /// # Arguments
    ///
    /// * `level` - The new minimum log level
    fn set_level(&mut self, level: LogLevel);

    /// Returns whether this handler is enabled.
    fn is_enabled(&self) -> bool;

    /// Sets whether this handler is enabled.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether the handler should be enabled
    fn set_enabled(&mut self, enabled: bool);
}

/// A simple handler that does nothing with log records.
/// Useful for testing or as a placeholder.
#[derive(Debug, Clone)]
pub struct NullHandler {
    level: LogLevel,
    enabled: bool,
}

impl NullHandler {
    /// Creates a new `NullHandler` with the given log level.
    ///
    /// # Arguments
    ///
    /// * `level` - The minimum log level to process
    pub fn new(level: LogLevel) -> Self {
        Self {
            level,
            enabled: true,
        }
    }
}

impl Handler for NullHandler {
    fn handle(&self, _record: &Record) -> bool {
        true
    }

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::Record;

    #[test]
    fn test_null_handler() {
        let mut handler = NullHandler::new(LogLevel::Info);
        let record = Record::new(LogLevel::Info, "Test message", "test_module", "test.rs", 42);

        assert!(handler.handle(&record));
        assert_eq!(handler.level(), LogLevel::Info);
        assert!(handler.is_enabled());

        handler.set_level(LogLevel::Debug);
        assert_eq!(handler.level(), LogLevel::Debug);

        handler.set_enabled(false);
        assert!(!handler.is_enabled());
    }
}
