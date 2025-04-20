use crate::level::LogLevel;
use crate::record::Record;
use std::sync::Arc;

/// Type alias for the format function
type FormatFn = Arc<dyn Fn(&Record) -> String + Send + Sync>;

/// A formatter for log records
#[derive(Clone)]
pub struct Formatter {
    /// Whether to use colors in the output
    pub use_colors: bool,
    /// Whether to include timestamps
    pub include_timestamp: bool,
    /// Whether to include the log level
    pub include_level: bool,
    /// Whether to include the module path
    pub include_module: bool,
    /// Whether to include the file and line number
    pub include_location: bool,
    /// The format pattern to use
    pub pattern: String,
    /// Custom format function
    format_fn: Option<FormatFn>,
}

impl std::fmt::Debug for Formatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Formatter")
            .field("use_colors", &self.use_colors)
            .field("include_timestamp", &self.include_timestamp)
            .field("include_level", &self.include_level)
            .field("include_module", &self.include_module)
            .field("include_location", &self.include_location)
            .field("pattern", &self.pattern)
            .field("format_fn", &self.format_fn.is_some())
            .finish()
    }
}

impl Formatter {
    /// Create a new formatter with default settings
    pub fn new() -> Self {
        Self {
            use_colors: true,
            include_timestamp: true,
            include_level: true,
            include_module: true,
            include_location: true,
            pattern: "{level} [{file}:{line}] {message} {metadata}".to_string(),
            format_fn: None,
        }
    }

    /// Set whether to use colors
    pub fn with_colors(mut self, use_colors: bool) -> Self {
        self.use_colors = use_colors;
        self
    }

    /// Set whether to include timestamps
    pub fn with_timestamp(mut self, include_timestamp: bool) -> Self {
        self.include_timestamp = include_timestamp;
        self
    }

    /// Set whether to include the log level
    pub fn with_level(mut self, include_level: bool) -> Self {
        self.include_level = include_level;
        self
    }

    /// Set whether to include the module path
    pub fn with_module(mut self, include_module: bool) -> Self {
        self.include_module = include_module;
        self
    }

    /// Set whether to include the file and line number
    pub fn with_location(mut self, include_location: bool) -> Self {
        self.include_location = include_location;
        self
    }

    /// Sets the format pattern to use.
    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = pattern.into();
        self.format_fn = None;
        self
    }

    /// Set a custom format function
    pub fn with_format<F>(mut self, format_fn: F) -> Self
    where
        F: Fn(&Record) -> String + Send + Sync + 'static,
    {
        self.format_fn = Some(Arc::new(format_fn));
        self
    }

    /// Format a log record
    pub fn format(&self, record: &Record) -> String {
        if let Some(format_fn) = &self.format_fn {
            return format_fn(record);
        }

        let mut output = String::new();

        if self.include_timestamp {
            output.push_str(
                &record
                    .timestamp()
                    .format("%Y-%m-%d %H:%M:%S%.3f")
                    .to_string(),
            );
            output.push(' ');
        }

        let mut formatted = String::new();

        if self.include_level {
            formatted.push_str(&record.level().to_string());
            formatted.push_str(" - ");
        }

        formatted.push_str(record.message());

        if self.include_module {
            formatted.push_str(&format!(" [{}]", record.module()));
        }

        if self.include_location {
            formatted.push_str(&format!(" ({}:{})", record.file(), record.line()));
        }

        if self.use_colors {
            formatted = format!(
                "{}{}{}",
                record.level().color(),
                formatted,
                LogLevel::reset_color()
            );
        }

        output.push_str(&formatted);

        if !record.metadata().is_empty() {
            output.push_str(" [");
            let mut keys: Vec<_> = record.metadata().keys().collect();
            keys.sort();
            let mut first = true;
            for key in keys {
                if !first {
                    output.push_str(", ");
                }
                output.push_str(&format!("{}={}", key, record.metadata().get(key).unwrap()));
                first = false;
            }
            output.push(']');
        }

        output
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formatter_creation() {
        let formatter = Formatter::new();
        assert!(formatter.use_colors);
        assert!(formatter.include_timestamp);
        assert!(formatter.include_level);
        assert!(formatter.include_module);
        assert!(formatter.include_location);
    }

    #[test]
    fn test_formatter_with_colors() {
        let formatter = Formatter::new().with_colors(false);
        assert!(!formatter.use_colors);
    }

    #[test]
    fn test_formatter_with_timestamp() {
        let formatter = Formatter::new().with_timestamp(false);
        assert!(!formatter.include_timestamp);
    }

    #[test]
    fn test_formatter_with_level() {
        let formatter = Formatter::new().with_level(false);
        assert!(!formatter.include_level);
    }

    #[test]
    fn test_formatter_with_module() {
        let formatter = Formatter::new().with_module(false);
        assert!(!formatter.include_module);
    }

    #[test]
    fn test_formatter_with_location() {
        let formatter = Formatter::new().with_location(false);
        assert!(!formatter.include_location);
    }

    #[test]
    fn test_formatter_format() {
        let formatter = Formatter::new();
        let record = Record::new(
            LogLevel::Info,
            "test message",
            None::<String>,
            None::<String>,
            None,
        );
        let formatted = formatter.format(&record);
        assert!(formatted.contains("test message"));
    }

    #[test]
    fn test_formatter_format_with_all_fields() {
        let formatter = Formatter::new();
        let record = Record::new(
            LogLevel::Info,
            "test message",
            Some("test_module".to_string()),
            Some("test_file.rs".to_string()),
            Some(42),
        );
        let formatted = formatter.format(&record);
        assert!(formatted.contains("test message"));
        assert!(formatted.contains("test_module"));
        assert!(formatted.contains("test_file.rs:42"));
    }

    #[test]
    fn test_formatter_format_with_colors() {
        let formatter = Formatter::new().with_colors(true);
        let record = Record::new(
            LogLevel::Error,
            "test message",
            None::<String>,
            None::<String>,
            None,
        );
        let formatted = formatter.format(&record);
        assert!(formatted.contains("\x1b[31m")); // Red color for Error
        assert!(formatted.contains("\x1b[0m")); // Reset color
    }

    #[test]
    fn test_formatter_format_without_colors() {
        let formatter = Formatter::new().with_colors(false);
        let record = Record::new(
            LogLevel::Error,
            "test message",
            None::<String>,
            None::<String>,
            None,
        );
        let formatted = formatter.format(&record);
        assert!(!formatted.contains("\x1b[31m")); // No red color
        assert!(!formatted.contains("\x1b[0m")); // No reset color
    }

    #[test]
    fn test_formatter_format_without_timestamp() {
        let formatter = Formatter::new().with_timestamp(false);
        let record = Record::new(
            LogLevel::Info,
            "test message",
            None::<String>,
            None::<String>,
            None,
        );
        let formatted = formatter.format(&record);
        assert!(!formatted.contains("2023")); // No year in timestamp
    }

    #[test]
    fn test_formatter_format_without_level() {
        let formatter = Formatter::new().with_level(false);
        let record = Record::new(
            LogLevel::Info,
            "test message",
            None::<String>,
            None::<String>,
            None,
        );
        let formatted = formatter.format(&record);
        assert!(!formatted.contains("INFO")); // No level
    }

    #[test]
    fn test_formatter_format_without_module() {
        let formatter = Formatter::new().with_module(false);
        let record = Record::new(
            LogLevel::Info,
            "test message",
            Some("test_module".to_string()),
            None::<String>,
            None,
        );
        let formatted = formatter.format(&record);
        assert!(!formatted.contains("test_module")); // No module
    }

    #[test]
    fn test_formatter_format_without_location() {
        let formatter = Formatter::new().with_location(false);
        let record = Record::new(
            LogLevel::Info,
            "test message",
            None::<String>,
            Some("test_file.rs".to_string()),
            Some(42),
        );
        let formatted = formatter.format(&record);
        assert!(!formatted.contains("test_file.rs:42")); // No location
    }

    #[test]
    fn test_formatter_format_long_message() {
        let formatter = Formatter::new();
        let long_message = "a".repeat(1000);
        let record = Record::new(
            LogLevel::Info,
            long_message.clone(),
            None::<String>,
            None::<String>,
            None,
        );
        let formatted = formatter.format(&record);
        assert!(formatted.contains(&long_message));
    }

    #[test]
    fn test_formatter_format_with_metadata() {
        let formatter = Formatter::new();
        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test_module".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        )
        .with_metadata("key1", "value1")
        .with_metadata("key2", "value2");

        let formatted = formatter.format(&record);
        assert!(formatted.contains("key1=value1"));
        assert!(formatted.contains("key2=value2"));
    }

    #[test]
    fn test_formatter_format_with_custom_format() {
        let formatter =
            Formatter::new().with_format(|record| format!("CUSTOM: {}", record.message()));
        let record = Record::new(
            LogLevel::Info,
            "test message",
            None::<String>,
            None::<String>,
            None,
        );
        let formatted = formatter.format(&record);
        assert_eq!(formatted, "CUSTOM: test message");
    }
}
