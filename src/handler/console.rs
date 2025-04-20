use std::io::{self, Write};

use crate::level::LogLevel;
use crate::record::Record;
use crate::formatter::Formatter;

use super::Handler;

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
    output: Box<dyn Write + Send + Sync>,
}

impl ConsoleHandler {
    /// Creates a new console handler that writes to stdout.
    pub fn stdout(level: LogLevel) -> Self {
        Self {
            level,
            enabled: true,
            formatter: Formatter::new(),
            output: Box::new(io::stdout()),
        }
    }

    /// Creates a new console handler that writes to stderr.
    pub fn stderr(level: LogLevel) -> Self {
        Self {
            level,
            enabled: true,
            formatter: Formatter::new(),
            output: Box::new(io::stderr()),
        }
    }

    /// Sets whether to use colors in the output.
    pub fn with_colors(mut self, use_colors: bool) -> Self {
        self.formatter = self.formatter.with_colors(use_colors);
        self
    }

    /// Sets the format pattern to use.
    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.formatter = self.formatter.with_pattern(pattern);
        self
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
        match writeln!(self.output, "{}", formatted) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::io::Cursor;

    struct TestOutput {
        buffer: Arc<Mutex<Vec<u8>>>,
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
            String::from_utf8(self.buffer.lock().unwrap().clone()).unwrap()
        }
    }

    #[test]
    fn test_console_handler_level_filtering() {
        let output = TestOutput::new();
        let mut handler = ConsoleHandler {
            level: LogLevel::Warning,
            enabled: true,
            formatter: Formatter::new().with_colors(false),
            output: Box::new(output),
        };

        let info_record = Record::new(
            LogLevel::Info,
            "Info message",
            "test_module",
            "test.rs",
            42,
        );
        let warning_record = Record::new(
            LogLevel::Warning,
            "Warning message",
            "test_module",
            "test.rs",
            42,
        );

        assert!(!handler.handle(&info_record));
        assert!(handler.handle(&warning_record));
    }

    #[test]
    fn test_console_handler_disabled() {
        let output = TestOutput::new();
        let mut handler = ConsoleHandler {
            level: LogLevel::Info,
            enabled: false,
            formatter: Formatter::new().with_colors(false),
            output: Box::new(output),
        };

        let record = Record::new(
            LogLevel::Info,
            "Test message",
            "test_module",
            "test.rs",
            42,
        );

        assert!(!handler.handle(&record));
        assert!(output.contents().is_empty());
    }

    #[test]
    fn test_console_handler_formatting() {
        let output = TestOutput::new();
        let mut handler = ConsoleHandler {
            level: LogLevel::Info,
            enabled: true,
            formatter: Formatter::new()
                .with_colors(false)
                .with_pattern("{level} - {message}"),
            output: Box::new(output),
        };

        let record = Record::new(
            LogLevel::Info,
            "Test message",
            "test_module",
            "test.rs",
            42,
        );

        assert!(handler.handle(&record));
        assert!(output.contents().contains("INFO - Test message"));
    }

    #[test]
    fn test_console_handler_colors() {
        let output = TestOutput::new();
        let mut handler = ConsoleHandler {
            level: LogLevel::Info,
            enabled: true,
            formatter: Formatter::new().with_colors(true),
            output: Box::new(output),
        };

        let record = Record::new(
            LogLevel::Error,
            "Error message",
            "test_module",
            "test.rs",
            42,
        );

        assert!(handler.handle(&record));
        let output = output.contents();
        assert!(output.contains("\x1b[31m")); // Red color for Error
        assert!(output.contains("\x1b[0m")); // Reset color
    }

    #[test]
    fn test_console_handler_metadata() {
        let output = TestOutput::new();
        let mut handler = ConsoleHandler {
            level: LogLevel::Info,
            enabled: true,
            formatter: Formatter::new().with_colors(false),
            output: Box::new(output),
        };

        let record = Record::new(
            LogLevel::Info,
            "Metadata test",
            "test_module",
            "test.rs",
            42,
        )
        .with_metadata("key1", "value1")
        .with_metadata("key2", "value2");

        assert!(handler.handle(&record));
        assert!(output.contents().contains("[key1=value1, key2=value2]"));
    }

    #[test]
    fn test_console_handler_structured_data() {
        let output = TestOutput::new();
        let mut handler = ConsoleHandler {
            level: LogLevel::Info,
            enabled: true,
            formatter: Formatter::new().with_colors(false),
            output: Box::new(output),
        };

        let data = serde_json::json!({
            "user_id": 123,
            "action": "login"
        });

        let record = Record::new(
            LogLevel::Info,
            "Structured data test",
            "test_module",
            "test.rs",
            42,
        )
        .with_structured_data("data", &data)
        .unwrap();

        assert!(handler.handle(&record));
        let output = output.contents();
        let formatted_json = output.split("data=").nth(1).unwrap()
            .trim_end_matches(']');
        let actual: serde_json::Value = serde_json::from_str(formatted_json).unwrap();
        assert_eq!(actual, data);
    }

    #[test]
    fn test_console_handler_write_error() {
        let mut output = Cursor::new(Vec::new());
        output.set_position(1); // Force write error
        let mut handler = ConsoleHandler {
            level: LogLevel::Info,
            enabled: true,
            formatter: Formatter::new().with_colors(false),
            output: Box::new(output),
        };

        let record = Record::new(
            LogLevel::Info,
            "Test message",
            "test_module",
            "test.rs",
            42,
        );

        assert!(!handler.handle(&record));
    }
} 