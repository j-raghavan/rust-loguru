use std::fmt;
use std::io::{self, Write};
use std::sync::Mutex;

use crate::formatters::Formatter;
use crate::level::LogLevel;
use crate::record::Record;

use super::Handler;

/// A wrapper around a writer that implements Debug
pub struct DebugWrite {
    writer: Mutex<Box<dyn Write + Send + Sync>>,
}

impl fmt::Debug for DebugWrite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DebugWrite")
            .field("writer", &"<writer>")
            .finish()
    }
}

/// A handler that writes to the console
#[derive(Debug)]
pub struct ConsoleHandler {
    /// The log level
    level: LogLevel,
    /// Whether the handler is enabled
    enabled: bool,
    /// The formatter to use
    formatter: Formatter,
    /// The output stream to write to
    output: DebugWrite,
}

impl Clone for ConsoleHandler {
    fn clone(&self) -> Self {
        Self {
            level: self.level,
            enabled: self.enabled,
            formatter: self.formatter.clone(),
            output: DebugWrite {
                writer: Mutex::new(Box::new(io::stdout())),
            },
        }
    }
}

impl ConsoleHandler {
    /// Create a new console handler that writes to stdout
    pub fn stdout(level: LogLevel) -> Self {
        Self {
            level,
            enabled: true,
            formatter: Formatter::text()
                .with_pattern("{level} - {message}")
                .with_colors(true),
            output: DebugWrite {
                writer: Mutex::new(Box::new(io::stdout())),
            },
        }
    }

    /// Create a new console handler that writes to stderr
    pub fn stderr(level: LogLevel) -> Self {
        Self {
            level,
            enabled: true,
            formatter: Formatter::text()
                .with_pattern("{level} - {message}")
                .with_colors(true),
            output: DebugWrite {
                writer: Mutex::new(Box::new(io::stderr())),
            },
        }
    }

    /// Create a new console handler with a custom writer
    pub fn with_writer(level: LogLevel, writer: Box<dyn Write + Send + Sync>) -> Self {
        Self {
            level,
            enabled: true,
            formatter: Formatter::text()
                .with_pattern("{level} - {message}")
                .with_colors(true),
            output: DebugWrite {
                writer: Mutex::new(writer),
            },
        }
    }

    /// Sets whether to use colors in the output.
    pub fn with_colors(mut self, use_colors: bool) -> Self {
        self.formatter = self.formatter.with_colors(use_colors);
        self
    }

    /// Sets a custom format pattern.
    pub fn with_pattern(self, pattern: impl Into<String>) -> Self {
        let mut handler = self;
        let formatter = handler.formatter.with_pattern(pattern);
        handler.formatter = formatter;
        handler
    }

    /// Sets a custom format function for the handler.
    pub fn with_format<F>(mut self, format_fn: F) -> Self
    where
        F: Fn(&Record) -> String + Send + Sync + 'static,
    {
        self.formatter = self.formatter.with_format(format_fn);
        self
    }

    pub fn with_formatter(mut self, formatter: Formatter) -> Self {
        self.formatter = formatter;
        self
    }
}

impl Default for ConsoleHandler {
    fn default() -> Self {
        Self::stdout(LogLevel::Info)
    }
}

impl Handler for ConsoleHandler {
    fn handle(&self, record: &Record) -> Result<(), String> {
        if !self.enabled || record.level() < self.level {
            return Ok(());
        }

        let formatted = self.formatter.format(record);
        let mut writer = self
            .output
            .writer
            .lock()
            .map_err(|e| format!("Failed to lock writer: {}", e))?;
        write!(writer, "{}", formatted)
            .map_err(|e| format!("Failed to write to console: {}", e))?;
        writer
            .flush()
            .map_err(|e| format!("Failed to flush console: {}", e))?;
        Ok(())
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

    fn formatter(&self) -> &Formatter {
        &self.formatter
    }

    fn set_formatter(&mut self, formatter: Formatter) {
        self.formatter = formatter;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct TestOutput {
        buffer: Arc<Mutex<Vec<u8>>>,
    }

    impl Clone for TestOutput {
        fn clone(&self) -> Self {
            Self {
                buffer: self.buffer.clone(),
            }
        }
    }

    impl Write for TestOutput {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.buffer.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl TestOutput {
        fn new() -> Self {
            Self {
                buffer: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn contents(&self) -> String {
            let buffer = self.buffer.lock().unwrap();
            String::from_utf8_lossy(&buffer).to_string()
        }
    }

    #[test]
    fn test_console_handler_level_filtering() {
        let output = TestOutput::new();
        let mut handler = ConsoleHandler::with_writer(LogLevel::Warning, Box::new(output.clone()));
        handler.set_level(LogLevel::Warning);

        let info_record = Record::new(
            LogLevel::Info,
            "info message",
            Some("test".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );
        let warning_record = Record::new(
            LogLevel::Warning,
            "warning message",
            Some("test".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        assert!(handler.handle(&info_record).is_ok());
        assert!(handler.handle(&warning_record).is_ok());
        assert!(output.contents().contains("warning message"));
    }

    #[test]
    fn test_console_handler_disabled() {
        let output = TestOutput::new();
        let mut handler = ConsoleHandler::with_writer(LogLevel::Warning, Box::new(output.clone()));
        handler.set_enabled(false);

        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        assert!(handler.handle(&record).is_ok());
        assert!(output.contents().is_empty());
    }

    #[test]
    fn test_console_handler_formatting() {
        let output = TestOutput::new();
        let handler = ConsoleHandler::with_writer(LogLevel::Info, Box::new(output.clone()))
            .with_pattern("{level} - {message}")
            .with_colors(false);

        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        assert!(handler.handle(&record).is_ok());
        assert!(output.contents().contains("INFO - Test message"));
    }

    #[test]
    fn test_console_handler_metadata() {
        let output = TestOutput::new();
        let handler = ConsoleHandler::with_writer(LogLevel::Info, Box::new(output.clone()));

        let mut record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );
        record = record.with_metadata("key1", "value1");
        record = record.with_metadata("key2", "value2");

        assert!(handler.handle(&record).is_ok());
        let contents = output.contents();
        assert!(contents.contains("key1=value1"));
        assert!(contents.contains("key2=value2"));
    }

    #[test]
    fn test_console_handler_structured_data() {
        let output = TestOutput::new();
        let handler = ConsoleHandler::with_writer(LogLevel::Info, Box::new(output.clone()))
            .with_pattern(r#"{{"level":"{level}","message":"{message}","module":"{module}"}}"#)
            .with_colors(false);

        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        assert!(handler.handle(&record).is_ok());
        let output = output.contents();
        assert!(output.contains(r#""level":"INFO""#));
        assert!(output.contains(r#""message":"Test message""#));
        assert!(output.contains(r#""module":"test""#));
    }

    #[test]
    fn test_handle_uses_configured_writer() {
        let output = TestOutput::new();
        let handler = ConsoleHandler::with_writer(LogLevel::Info, Box::new(output.clone()));
        let record = Record::new(
            LogLevel::Info,
            "test message",
            Some("test".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        assert!(handler.handle(&record).is_ok());
        assert!(output.contents().contains("test message"));
    }

    #[test]
    fn test_handle_respects_disabled() {
        let output = TestOutput::new();
        let mut handler = ConsoleHandler::with_writer(LogLevel::Info, Box::new(output.clone()));
        handler.set_enabled(false);
        let record = Record::new(
            LogLevel::Info,
            "test message",
            Some("test".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        assert!(handler.handle(&record).is_ok());
        assert!(output.contents().is_empty());
    }

    #[test]
    fn test_handle_respects_level() {
        let output = TestOutput::new();
        let mut handler = ConsoleHandler::with_writer(LogLevel::Info, Box::new(output.clone()));
        handler.set_level(LogLevel::Error);
        let record = Record::new(
            LogLevel::Info,
            "test message",
            Some("test".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        assert!(handler.handle(&record).is_ok());
        assert!(output.contents().is_empty());
    }
}
