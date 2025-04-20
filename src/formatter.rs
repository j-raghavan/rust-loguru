use crate::level::LogLevel;
use crate::record::Record;

/// A formatter for log records
#[derive(Debug, Clone)]
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

    /// Format a log record
    pub fn format(&self, record: &Record) -> String {
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

        if self.include_level {
            let level_str = if self.use_colors {
                format!(
                    "{}{}{}",
                    record.level().color(),
                    record.level(),
                    LogLevel::reset_color()
                )
            } else {
                record.level().to_string()
            };
            output.push_str(&level_str);
            output.push(' ');
        }

        if self.include_module {
            output.push_str(&format!("[{}] ", record.module()));
        }

        output.push_str(record.message());

        if self.include_location {
            output.push_str(&format!(" ({}:{})", record.file(), record.line()));
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
        assert!(formatted.contains("test_file.rs"));
        assert!(formatted.contains("42"));
    }

    #[test]
    fn test_formatter_format_with_colors() {
        let formatter = Formatter::new();
        let record = Record::new(
            LogLevel::Error,
            "test message",
            Some("test_module".to_string()),
            Some("test_file.rs".to_string()),
            Some(42),
        );
        let formatted = formatter.format(&record);
        assert!(formatted.contains("\x1b[31m")); // Red color for error
        assert!(formatted.contains("\x1b[0m")); // Reset color
    }

    #[test]
    fn test_formatter_format_without_colors() {
        let formatter = Formatter::new().with_colors(false);
        let record = Record::new(
            LogLevel::Error,
            "test message",
            Some("test_module".to_string()),
            Some("test_file.rs".to_string()),
            Some(42),
        );
        let formatted = formatter.format(&record);
        assert!(!formatted.contains("\x1b[31m")); // No color codes
        assert!(!formatted.contains("\x1b[0m"));
    }

    #[test]
    fn test_formatter_format_without_timestamp() {
        let formatter = Formatter::new().with_timestamp(false);
        let record = Record::new(
            LogLevel::Info,
            "test message",
            Some("test_module".to_string()),
            Some("test_file.rs".to_string()),
            Some(42),
        );
        let formatted = formatter.format(&record);
        assert!(!formatted.contains("2024")); // No year in timestamp
    }

    #[test]
    fn test_formatter_format_without_level() {
        let formatter = Formatter::new().with_level(false);
        let record = Record::new(
            LogLevel::Info,
            "test message",
            Some("test_module".to_string()),
            Some("test_file.rs".to_string()),
            Some(42),
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
            Some("test_file.rs".to_string()),
            Some(42),
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
            Some("test_module".to_string()),
            Some("test_file.rs".to_string()),
            Some(42),
        );
        let formatted = formatter.format(&record);
        assert!(!formatted.contains("test_file.rs")); // No file
        assert!(!formatted.contains("42")); // No line number
    }

    #[test]
    fn test_formatter_format_long_message() {
        let formatter = Formatter::new();
        let long_message = "a".repeat(1000);
        let record = Record::new(
            LogLevel::Info,
            &long_message,
            Some("test_module".to_string()),
            Some("test_file.rs".to_string()),
            Some(42),
        );
        let formatted = formatter.format(&record);
        assert!(formatted.contains(&long_message));
    }
}
