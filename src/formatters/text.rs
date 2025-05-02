use colored::Colorize;
use std::fmt;

use crate::formatters::FormatFn;
use crate::formatters::FormatterTrait;
use crate::level::LogLevel;
use crate::record::Record;

/// A text formatter that formats log records as text
#[derive(Clone)]
pub struct TextFormatter {
    /// Whether to use colors in the output
    use_colors: bool,
    /// Whether to include timestamps in the output
    include_timestamp: bool,
    /// Whether to include log levels in the output
    include_level: bool,
    /// Whether to include module names in the output
    include_module: bool,
    /// Whether to include file locations in the output
    include_location: bool,
    /// A custom format function
    format_fn: Option<FormatFn>,
}

impl fmt::Debug for TextFormatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextFormatter")
            .field("use_colors", &self.use_colors)
            .field("include_timestamp", &self.include_timestamp)
            .field("include_level", &self.include_level)
            .field("include_module", &self.include_module)
            .field("include_location", &self.include_location)
            .field("format_fn", &"<format_fn>")
            .finish()
    }
}

impl Default for TextFormatter {
    fn default() -> Self {
        Self {
            use_colors: true,
            include_timestamp: true,
            include_level: true,
            include_module: true,
            include_location: true,
            format_fn: None,
        }
    }
}

impl FormatterTrait for TextFormatter {
    fn fmt(&self, record: &Record) -> String {
        // If a custom format function is provided, use it
        if let Some(format_fn) = &self.format_fn {
            return format_fn(record);
        }

        let mut result = String::new();

        if self.include_timestamp {
            result.push_str(&record.timestamp().to_rfc3339());
            result.push(' ');
        }

        if self.include_level {
            let level_str = record.level().to_string();
            if self.use_colors {
                result.push_str(&match record.level() {
                    LogLevel::Trace => level_str.white().to_string(),
                    LogLevel::Debug => level_str.blue().to_string(),
                    LogLevel::Info => level_str.green().to_string(),
                    LogLevel::Warning => level_str.yellow().to_string(),
                    LogLevel::Error => level_str.red().to_string(),
                    LogLevel::Critical => level_str.red().bold().to_string(),
                    LogLevel::Success => level_str.green().bold().to_string(),
                });
            } else {
                result.push_str(&level_str);
            }
            result.push_str(" - ");
        }

        if self.include_module {
            let module = record.module();
            if module != "unknown" {
                result.push_str(module);
                result.push(' ');
            }
        }

        if self.include_location {
            // Only include file and line, not module
            let file = record.file();
            let line = record.line();
            if file != "unknown" {
                result.push_str(&format!("{}:{}", file, line));
                result.push(' ');
            }
        }

        result.push_str(record.message());

        if !result.ends_with('\n') {
            result.push('\n');
        }

        result
    }

    fn with_colors(&mut self, use_colors: bool) {
        self.use_colors = use_colors;
    }

    fn with_timestamp(&mut self, include_timestamp: bool) {
        self.include_timestamp = include_timestamp;
    }

    fn with_level(&mut self, include_level: bool) {
        self.include_level = include_level;
    }

    fn with_module(&mut self, include_module: bool) {
        self.include_module = include_module;
    }

    fn with_location(&mut self, include_location: bool) {
        self.include_location = include_location;
    }

    fn with_pattern(&mut self, _pattern: String) {
        // Text formatter doesn't use patterns
    }

    fn with_format(&mut self, format_fn: FormatFn) {
        self.format_fn = Some(format_fn);
    }

    fn box_clone(&self) -> Box<dyn FormatterTrait + Send + Sync> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::LogLevel;

    #[test]
    fn test_text_formatter_default() {
        let formatter = TextFormatter::default();
        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        let formatted = FormatterTrait::fmt(&formatter, &record);
        assert!(formatted.contains("Test message"));
        assert!(formatted.contains("INFO"));
        assert!(formatted.contains("test"));
        assert!(formatted.contains("test.rs:42"));
    }

    #[test]
    fn test_text_formatter_no_colors() {
        let mut formatter = TextFormatter::default();
        formatter.with_colors(false);
        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        let formatted = FormatterTrait::fmt(&formatter, &record);
        assert!(formatted.contains("Test message"));
        assert!(formatted.contains("INFO"));
        assert!(formatted.contains("test"));
        assert!(formatted.contains("test.rs:42"));
        assert!(!formatted.contains("\x1b["));
    }

    #[test]
    fn test_text_formatter_no_timestamp() {
        let mut formatter = TextFormatter::default();
        formatter.with_timestamp(false);
        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        let formatted = FormatterTrait::fmt(&formatter, &record);
        assert!(formatted.contains("Test message"));
        assert!(formatted.contains("INFO"));
        assert!(formatted.contains("test"));
        assert!(formatted.contains("test.rs:42"));
        assert!(!formatted.contains("2023")); // No year in timestamp
    }

    #[test]
    fn test_text_formatter_no_level() {
        let mut formatter = TextFormatter::default();
        formatter.with_level(false);
        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        let formatted = FormatterTrait::fmt(&formatter, &record);
        assert!(formatted.contains("Test message"));
        assert!(!formatted.contains("INFO"));
        assert!(formatted.contains("test"));
        assert!(formatted.contains("test.rs:42"));
    }

    #[test]
    fn test_text_formatter_no_module() {
        let mut formatter = TextFormatter::default();
        formatter.with_module(false);
        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test_module".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        let formatted = FormatterTrait::fmt(&formatter, &record);
        assert!(formatted.contains("Test message"));
        assert!(formatted.contains("INFO"));
        assert!(!formatted.contains("test_module"));
        assert!(formatted.contains("test.rs:42"));
    }

    #[test]

    fn test_text_formatter_custom_format() {
        use std::sync::Arc;
        let mut formatter = TextFormatter::default();
        formatter.with_format(Arc::new(|record: &Record| {
            "CUSTOM: ".to_string() + record.message()
        }));
        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        let formatted = FormatterTrait::fmt(&formatter, &record);
        assert_eq!(formatted, "CUSTOM: Test message");
    }
}
