use std::io::{self, Write};

use crate::formatter::Formatter;
use crate::level::LogLevel;
use crate::record::Record;

use super::Handler;

/// A wrapper around a writer that implements Debug
pub struct DebugWrite {
    writer: Box<dyn Write + Send + Sync>,
}

impl std::fmt::Debug for DebugWrite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebugWrite")
            .field("writer", &"<dyn Write>")
            .finish()
    }
}

/// A handler that writes log records to the console.
#[derive(Debug)]
pub struct ConsoleHandler {
    /// The minimum log level to handle
    level: LogLevel,
    /// Whether the handler is enabled
    enabled: bool,
    /// The formatter to use
    formatter: Formatter,
    /// The output stream to write to
    output: DebugWrite,
}

impl ConsoleHandler {
    /// Creates a new console handler that writes to stdout.
    pub fn new() -> Self {
        Self::stdout(LogLevel::Info)
    }

    /// Creates a new console handler that writes to stdout.
    pub fn stdout(level: LogLevel) -> Self {
        Self {
            level,
            enabled: true,
            formatter: Formatter::new(),
            output: DebugWrite {
                writer: Box::new(io::stdout()),
            },
        }
    }

    /// Creates a new console handler that writes to stderr.
    pub fn stderr(level: LogLevel) -> Self {
        Self {
            level,
            enabled: true,
            formatter: Formatter::new(),
            output: DebugWrite {
                writer: Box::new(io::stderr()),
            },
        }
    }

    /// Creates a new console handler with a custom writer.
    pub fn with_writer(writer: Box<dyn Write + Send + Sync>) -> Self {
        Self {
            level: LogLevel::Info,
            enabled: true,
            formatter: Formatter::new(),
            output: DebugWrite { writer },
        }
    }

    /// Sets whether to use colors in the output.
    pub fn with_colors(mut self, use_colors: bool) -> Self {
        self.formatter = self.formatter.with_colors(use_colors);
        self
    }

    /// Sets a custom format pattern.
    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.formatter = self.formatter.with_pattern(pattern);
        self
    }

    /// Sets a custom format function for the handler.
    pub fn with_format<F>(mut self, format_fn: F) -> Self
    where
        F: Fn(&Record) -> String + Send + Sync + 'static,
    {
        self.formatter = self.formatter.with_format(format_fn);
        self
    }
}

impl Default for ConsoleHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Handler for ConsoleHandler {
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
        if !self.enabled || record.level() < self.level {
            return false;
        }

        let formatted = self.formatter.format(record);
        if writeln!(self.output.writer, "{}", formatted).is_err() {
            return false;
        }

        true
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
        let mut handler = ConsoleHandler::with_writer(Box::new(output.clone()));
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

        assert!(!handler.handle(&info_record));
        assert!(handler.handle(&warning_record));
        assert!(output.contents().contains("warning message"));
    }

    #[test]
    fn test_console_handler_disabled() {
        let output = TestOutput::new();
        let mut handler = ConsoleHandler::with_writer(Box::new(output.clone()));
        handler.set_enabled(false);

        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        assert!(!handler.handle(&record));
        assert!(output.contents().is_empty());
    }

    #[test]
    fn test_console_handler_formatting() {
        let output = TestOutput::new();
        let mut handler = ConsoleHandler::with_writer(Box::new(output.clone()));

        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        assert!(handler.handle(&record));
        assert!(output.contents().contains("INFO - Test message"));
    }

    #[test]
    fn test_console_handler_colors() {
        let output = TestOutput::new();
        let mut handler = ConsoleHandler::with_writer(Box::new(output.clone())).with_colors(true);

        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        assert!(handler.handle(&record));
        let output = output.contents();
        assert!(output.contains("\x1b["));
    }

    #[test]
    fn test_console_handler_metadata() {
        let output = TestOutput::new();
        let mut handler = ConsoleHandler::with_writer(Box::new(output.clone()));

        let mut record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );
        record = record.with_metadata("key1", "value1");
        record = record.with_metadata("key2", "value2");

        assert!(handler.handle(&record));
        let contents = output.contents();
        assert!(contents.contains("key1=value1"));
        assert!(contents.contains("key2=value2"));
    }

    #[test]
    fn test_console_handler_structured_data() {
        let output = TestOutput::new();
        let mut handler =
            ConsoleHandler::with_writer(Box::new(output.clone())).with_format(|record| {
                format!(
                    r#"{{"level":"{}","message":"{}","module":"{}"}}"#,
                    record.level(),
                    record.message(),
                    record.module()
                )
            });

        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        assert!(handler.handle(&record));
        let output = output.contents();
        assert!(output.contains(r#""level":"INFO""#));
        assert!(output.contains(r#""message":"Test message""#));
        assert!(output.contains(r#""module":"test""#));
    }
}
