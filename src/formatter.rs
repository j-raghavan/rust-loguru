use crate::formatters::FormatterTrait;
use crate::level::LogLevel;
use crate::record::Record;
use chrono::Local;
use colored::*;
use std::fmt;
use std::sync::Arc;

/// A type alias for a format function
pub type FormatFn = Arc<dyn Fn(&Record) -> String + Send + Sync>;

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
    /// The format pattern to use
    pattern: String,
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
            .field("pattern", &self.pattern)
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
            pattern: "{timestamp} {level} {module} {location} {message}".to_string(),
            format_fn: None,
        }
    }
}

impl FormatterTrait for TextFormatter {
    fn fmt(&self, record: &Record) -> String {
        if let Some(format_fn) = &self.format_fn {
            return format_fn(record);
        }

        let mut output = self.pattern.clone();

        if self.include_timestamp {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            output = output.replace("{timestamp}", &timestamp.to_string());
        } else {
            output = output.replace("{timestamp}", "");
        }

        if self.include_level {
            let level = record.level().to_string();
            if self.use_colors {
                let colored_level = match record.level() {
                    LogLevel::Error => level.red().to_string(),
                    LogLevel::Warning => level.yellow().to_string(),
                    LogLevel::Info => level.green().to_string(),
                    LogLevel::Debug => level.blue().to_string(),
                    LogLevel::Trace => level.cyan().to_string(),
                    LogLevel::Success => level.green().to_string(),
                    LogLevel::Critical => level.red().to_string(),
                };
                output = output.replace("{level}", &colored_level);
            } else {
                output = output.replace("{level}", &level);
            }
        } else {
            output = output.replace("{level}", "");
        }

        if self.include_module {
            output = output.replace("{module}", record.module());
        } else {
            output = output.replace("{module}", "");
        }

        if self.include_location {
            let location = format!("{}:{}", record.file(), record.line());
            output = output.replace("{location}", &location);
        } else {
            output = output.replace("{location}", "");
        }

        output = output.replace("{message}", record.message());

        output.trim().to_string()
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

    fn with_pattern(&mut self, pattern: String) {
        self.pattern = pattern;
    }

    fn with_format(&mut self, format_fn: FormatFn) {
        self.format_fn = Some(format_fn);
    }

    fn box_clone(&self) -> Box<dyn FormatterTrait + Send + Sync> {
        Box::new(Self {
            use_colors: self.use_colors,
            include_timestamp: self.include_timestamp,
            include_level: self.include_level,
            include_module: self.include_module,
            include_location: self.include_location,
            pattern: self.pattern.clone(),
            format_fn: self.format_fn.clone(),
        })
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
        let mut formatter = TextFormatter::default();
        formatter.with_format(Arc::new(|record: &Record| {
            format!("CUSTOM: {}", record.message())
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
