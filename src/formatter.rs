use chrono::{DateTime, Utc};
use std::collections::BTreeMap;

use crate::level::LogLevel;
use crate::record::Record;

/// Represents a formatter for log records.
#[derive(Debug, Clone)]
pub struct Formatter {
    /// Whether to use colors in the output
    use_colors: bool,
    /// The format pattern to use
    pattern: String,
}

impl Formatter {
    /// Creates a new formatter with default settings.
    ///
    /// The default format is: `[timestamp] level file:line - message`
    pub fn new() -> Self {
        Self {
            use_colors: true,
            pattern: String::from("[{timestamp}] {level} {file}:{line} - {message}"),
        }
    }

    /// Sets whether to use colors in the output.
    pub fn with_colors(mut self, use_colors: bool) -> Self {
        self.use_colors = use_colors;
        self
    }

    /// Sets the format pattern to use.
    ///
    /// Available placeholders:
    /// - `{timestamp}` - The log timestamp
    /// - `{level}` - The log level
    /// - `{message}` - The log message
    /// - `{file}` - The source file
    /// - `{line}` - The line number
    /// - `{module}` - The module name
    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = pattern.into();
        self
    }

    /// Formats a log record according to the configured pattern.
    pub fn format(&self, record: &Record) -> String {
        let mut result = self.pattern.clone();

        // Replace placeholders with actual values
        result = result.replace("{timestamp}", &format_timestamp(record.timestamp()));
        result = result.replace("{level}", &format_level(record.level(), self.use_colors));
        result = result.replace("{message}", record.message());
        result = result.replace("{file}", record.file());
        result = result.replace("{line}", &record.line().to_string());
        result = result.replace("{module}", record.module());

        // Add metadata if present
        if !record.metadata().is_empty() {
            result.push_str(" [");
            // Use BTreeMap to ensure consistent ordering
            let metadata: BTreeMap<_, _> = record.metadata().iter().collect();
            let metadata_str = metadata
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(", ");
            result.push_str(&metadata_str);
            result.push(']');
        }

        result
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Formats a timestamp according to the standard format.
fn format_timestamp(timestamp: DateTime<Utc>) -> String {
    timestamp.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
}

/// Formats a log level with optional color.
fn format_level(level: LogLevel, use_colors: bool) -> String {
    if use_colors {
        format!("{}{}{}", level.color(), level, LogLevel::reset_color())
    } else {
        level.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_formatter_default() {
        let formatter = Formatter::new();
        let record = Record::new(LogLevel::Info, "Test message", "test_module", "test.rs", 42);

        let formatted = formatter.format(&record);
        assert!(formatted.contains("INFO"));
        assert!(formatted.contains("test.rs:42"));
        assert!(formatted.contains("Test message"));
    }

    #[test]
    fn test_formatter_custom_pattern() {
        let formatter = Formatter::new()
            .with_pattern("{level} - {message} [{file}:{line}]")
            .with_colors(false);
        let record = Record::new(
            LogLevel::Error,
            "Custom format test",
            "test_module",
            "test.rs",
            42,
        );

        let formatted = formatter.format(&record);
        assert!(formatted.starts_with("ERROR"));
        assert!(formatted.contains("Custom format test"));
        assert!(formatted.ends_with("[test.rs:42]"));
    }

    #[test]
    fn test_formatter_without_colors() {
        let formatter = Formatter::new().with_colors(false);
        let record = Record::new(
            LogLevel::Warning,
            "No colors test",
            "test_module",
            "test.rs",
            42,
        );

        let formatted = formatter.format(&record);
        assert!(!formatted.contains("\x1b[")); // No ANSI color codes
        assert!(formatted.contains("WARNING"));
    }

    #[test]
    fn test_formatter_with_metadata() {
        let formatter = Formatter::new().with_colors(false);
        let record = Record::new(
            LogLevel::Info,
            "Metadata test",
            "test_module",
            "test.rs",
            42,
        )
        .with_metadata("key1", "value1")
        .with_metadata("key2", "value2");

        let formatted = formatter.format(&record);
        assert!(formatted.contains("[key1=value1, key2=value2]"));
    }

    #[test]
    fn test_formatter_timestamp_format() {
        let formatter = Formatter::new();
        let record = Record::new(
            LogLevel::Info,
            "Timestamp test",
            "test_module",
            "test.rs",
            42,
        );

        let formatted = formatter.format(&record);
        let timestamp_str = record
            .timestamp()
            .format("%Y-%m-%d %H:%M:%S%.3f")
            .to_string();
        assert!(formatted.contains(&timestamp_str));
    }

    #[test]
    fn test_formatter_level_colors() {
        let formatter = Formatter::new();
        let record = Record::new(LogLevel::Error, "Color test", "test_module", "test.rs", 42);

        let formatted = formatter.format(&record);
        assert!(formatted.contains("\x1b[31m")); // Red color for Error
        assert!(formatted.contains("\x1b[0m")); // Reset color
    }

    #[test]
    fn test_formatter_empty_pattern() {
        let formatter = Formatter::new().with_pattern("");
        let record = Record::new(
            LogLevel::Info,
            "Empty pattern test",
            "test_module",
            "test.rs",
            42,
        );

        let formatted = formatter.format(&record);
        assert_eq!(formatted, "");
    }

    #[test]
    fn test_formatter_unknown_placeholder() {
        let formatter = Formatter::new()
            .with_pattern("{unknown} {level}")
            .with_colors(false);
        let record = Record::new(
            LogLevel::Info,
            "Unknown placeholder test",
            "test_module",
            "test.rs",
            42,
        );

        let formatted = formatter.format(&record);
        assert_eq!(formatted, "{unknown} INFO");
    }

    #[test]
    fn test_formatter_all_placeholders() {
        let formatter = Formatter::new()
            .with_pattern("{timestamp} {level} [{module}] {file}:{line} - {message}")
            .with_colors(false);
        let record = Record::new(
            LogLevel::Info,
            "All placeholders test",
            "test_module",
            "test.rs",
            42,
        );

        let formatted = formatter.format(&record);
        assert!(formatted.contains("test_module"));
        assert!(formatted.contains("test.rs:42"));
        assert!(formatted.contains("All placeholders test"));
        assert!(formatted.contains("INFO"));
    }

    #[test]
    fn test_formatter_repeated_placeholders() {
        let formatter = Formatter::new()
            .with_pattern("{level} {message} - Level: {level}")
            .with_colors(false);
        let record = Record::new(
            LogLevel::Warning,
            "Repeated test",
            "test_module",
            "test.rs",
            42,
        );

        let formatted = formatter.format(&record);
        assert_eq!(formatted, "WARNING Repeated test - Level: WARNING");
    }

    #[test]
    fn test_formatter_with_structured_metadata() {
        let formatter = Formatter::new().with_colors(false);
        let data = json!({
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

        let formatted = formatter.format(&record);
        // Parse both the expected and actual JSON to compare them
        let formatted_json = formatted
            .split("data=")
            .nth(1)
            .unwrap()
            .trim_end_matches(']');
        let actual: serde_json::Value = serde_json::from_str(formatted_json).unwrap();
        let expected = json!({
            "user_id": 123,
            "action": "login"
        });
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_formatter_with_multiple_metadata_types() {
        let formatter = Formatter::new().with_colors(false);
        let record = Record::new(
            LogLevel::Info,
            "Multiple metadata types",
            "test_module",
            "test.rs",
            42,
        )
        .with_metadata("string", "value")
        .with_metadata("number", "42")
        .with_metadata("boolean", "true");

        let formatted = formatter.format(&record);
        assert!(formatted.contains("string=value"));
        assert!(formatted.contains("number=42"));
        assert!(formatted.contains("boolean=true"));
    }

    #[test]
    fn test_formatter_with_special_characters() {
        let formatter = Formatter::new().with_colors(false);
        let record = Record::new(
            LogLevel::Info,
            "Message with special chars: !@#$%^&*()",
            "test_module",
            "test.rs",
            42,
        )
        .with_metadata("key!@#", "value$%^");

        let formatted = formatter.format(&record);
        assert!(formatted.contains("Message with special chars: !@#$%^&*()"));
        assert!(formatted.contains("key!@#=value$%^"));
    }

    #[test]
    fn test_formatter_with_multiline_message() {
        let formatter = Formatter::new().with_colors(false);
        let record = Record::new(
            LogLevel::Info,
            "Line 1\nLine 2\nLine 3",
            "test_module",
            "test.rs",
            42,
        );

        let formatted = formatter.format(&record);
        assert!(formatted.contains("Line 1\nLine 2\nLine 3"));
    }

    #[test]
    fn test_formatter_with_unicode() {
        let formatter = Formatter::new().with_colors(false);
        let record = Record::new(
            LogLevel::Info,
            "Unicode test: 🦀 💻 📝",
            "test_module",
            "test.rs",
            42,
        )
        .with_metadata("emoji", "🎯");

        let formatted = formatter.format(&record);
        assert!(formatted.contains("🦀 💻 📝"));
        assert!(formatted.contains("emoji=🎯"));
    }

    #[test]
    fn test_formatter_with_long_values() {
        let formatter = Formatter::new().with_colors(false);
        let long_message = "a".repeat(1000);
        let record = Record::new(LogLevel::Info, &long_message, "test_module", "test.rs", 42)
            .with_metadata("long_key", &"b".repeat(100));

        let formatted = formatter.format(&record);
        assert!(formatted.contains(&long_message));
        assert!(formatted.contains(&"b".repeat(100)));
    }

    #[test]
    fn test_formatter_all_log_levels() {
        let formatter = Formatter::new().with_colors(false);
        let levels = vec![
            LogLevel::Trace,
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warning,
            LogLevel::Error,
            LogLevel::Critical,
        ];

        for level in levels {
            let record = Record::new(level, "Test message", "test_module", "test.rs", 42);
            let formatted = formatter.format(&record);
            assert!(formatted.contains(&level.to_string()));
        }
    }
}
