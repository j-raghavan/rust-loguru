use crate::formatters::{FormatFn, FormatterTrait};
use crate::record::Record;
use serde_json;
use serde_json::json;
use std::fmt;
use std::sync::Arc;

/// A JSON formatter that formats log records as JSON
#[derive(Clone)]
pub struct JsonFormatter {
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

impl fmt::Debug for JsonFormatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JsonFormatter")
            .field("use_colors", &self.use_colors)
            .field("include_timestamp", &self.include_timestamp)
            .field("include_level", &self.include_level)
            .field("include_module", &self.include_module)
            .field("include_location", &self.include_location)
            .field("format_fn", &"<format_fn>")
            .finish()
    }
}

impl Default for JsonFormatter {
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

impl JsonFormatter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_colors(self, _use_colors: bool) -> Self {
        self
    }

    pub fn with_timestamp(mut self, include_timestamp: bool) -> Self {
        self.include_timestamp = include_timestamp;
        self
    }

    pub fn with_level(mut self, include_level: bool) -> Self {
        self.include_level = include_level;
        self
    }

    pub fn with_module(mut self, include_module: bool) -> Self {
        self.include_module = include_module;
        self
    }

    pub fn with_location(mut self, include_location: bool) -> Self {
        self.include_location = include_location;
        self
    }

    pub fn with_format<F>(mut self, format_fn: F) -> Self
    where
        F: Fn(&Record) -> String + Send + Sync + 'static,
    {
        self.format_fn = Some(Arc::new(format_fn));
        self
    }
}

impl FormatterTrait for JsonFormatter {
    fn fmt(&self, record: &Record) -> String {
        if let Some(format_fn) = &self.format_fn {
            return format_fn(record);
        }

        let mut json = json!({
            "message": record.message(),
        });

        if self.include_timestamp {
            json["timestamp"] = json!(record.timestamp().to_rfc3339());
        }
        if self.include_level {
            json["level"] = json!(record.level().to_string());
        }
        if self.include_module {
            json["module"] = json!(record.module());
        }
        if self.include_location {
            json["location"] = json!(record.location().to_string());
        }

        json.to_string()
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
        // JSON formatter doesn't use patterns
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
    fn test_json_formatter_default() {
        let formatter = JsonFormatter::default();
        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        let formatted = formatter.fmt(&record);
        assert!(formatted.contains("Test message"));
        assert!(formatted.contains("INFO"));
        assert!(formatted.contains("test"));
        assert!(formatted.contains("test.rs:42"));
    }

    #[test]
    fn test_json_formatter_no_timestamp() {
        let formatter = JsonFormatter::default().with_timestamp(false);
        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        let formatted = formatter.fmt(&record);
        assert!(formatted.contains("Test message"));
        assert!(formatted.contains("INFO"));
        assert!(formatted.contains("test"));
        assert!(formatted.contains("test.rs:42"));
        assert!(!formatted.contains("timestamp"));
    }

    #[test]
    fn test_json_formatter_no_level() {
        let formatter = JsonFormatter::default().with_level(false);
        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        let formatted = formatter.fmt(&record);
        assert!(formatted.contains("Test message"));
        assert!(!formatted.contains("INFO"));
        assert!(formatted.contains("test"));
        assert!(formatted.contains("test.rs:42"));
    }

    #[test]
    fn test_json_formatter_no_module() {
        let formatter = JsonFormatter::default().with_module(false);
        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test".to_string()),
            Some("main.rs".to_string()),
            Some(42),
        );

        let formatted = formatter.fmt(&record);
        assert!(formatted.contains("Test message"));
        assert!(formatted.contains("INFO"));
        assert!(!formatted.contains("test"));
        assert!(formatted.contains("main.rs:42"));
    }

    #[test]
    fn test_json_formatter_no_location() {
        let formatter = JsonFormatter::default().with_location(false);
        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        let formatted = formatter.fmt(&record);
        assert!(formatted.contains("Test message"));
        assert!(formatted.contains("INFO"));
        assert!(formatted.contains("test"));
    }

    #[test]
    fn test_json_formatter_custom_format() {
        let formatter =
            JsonFormatter::default().with_format(|record| format!("CUSTOM: {}", record.message()));
        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        let formatted = formatter.fmt(&record);
        assert_eq!(formatted, "CUSTOM: Test message");
    }
}
