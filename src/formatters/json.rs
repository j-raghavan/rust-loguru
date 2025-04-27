use crate::formatters::FormatterTrait;
use crate::record::Record;
use chrono::Local;
use serde_json::json;
use std::fmt;
use std::sync::Arc;

/// A type alias for a format function
pub type FormatFn = Arc<dyn Fn(&Record) -> String + Send + Sync>;

/// A JSON formatter that formats log records as JSON
#[derive(Clone)]
pub struct JsonFormatter {
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

impl fmt::Debug for JsonFormatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JsonFormatter")
            .field("include_timestamp", &self.include_timestamp)
            .field("include_level", &self.include_level)
            .field("include_module", &self.include_module)
            .field("include_location", &self.include_location)
            .field("pattern", &self.pattern)
            .field("format_fn", &"<format_fn>")
            .finish()
    }
}

impl Default for JsonFormatter {
    fn default() -> Self {
        Self {
            include_timestamp: true,
            include_level: true,
            include_module: true,
            include_location: true,
            pattern: "{timestamp} {level} {module} {location} {message}".to_string(),
            format_fn: None,
        }
    }
}

impl JsonFormatter {
    pub fn new() -> Self {
        Self::default()
    }
}

impl FormatterTrait for JsonFormatter {
    fn format(&self, record: &Record) -> String {
        if let Some(format_fn) = &self.format_fn {
            return format_fn(record);
        }

        // If a custom pattern is provided and it's not the default pattern, use it
        if self.pattern != "{timestamp} {level} {module} {location} {message}" {
            let mut result = self.pattern.clone();

            // Replace placeholders with JSON-formatted values
            if self.include_level {
                result = result.replace("{level}", &record.level().to_string());
            } else {
                result = result.replace("{level}", "");
            }

            if self.include_module {
                result = result.replace("{module}", record.module());
            } else {
                result = result.replace("{module}", "");
            }

            if self.include_location {
                result = result.replace(
                    "{location}",
                    &format!("{}:{}", record.file(), record.line()),
                );
            } else {
                result = result.replace("{location}", "");
            }

            if self.include_timestamp {
                let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
                result = result.replace("{timestamp}", &timestamp.to_string());
            } else {
                result = result.replace("{timestamp}", "");
            }

            result = result.replace("{message}", record.message());

            // Ensure newline at end
            if result.ends_with('\n') {
                result.to_string()
            } else {
                format!("{}\n", result)
            }
        } else {
            // Use default JSON formatting
            let mut json = serde_json::Map::new();

            if self.include_timestamp {
                let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
                json.insert("timestamp".to_string(), json!(timestamp.to_string()));
            }

            if self.include_level {
                json.insert("level".to_string(), json!(record.level().to_string()));
            }

            if self.include_module {
                json.insert("module".to_string(), json!(record.module()));
            }

            if self.include_location {
                json.insert(
                    "location".to_string(),
                    json!(format!("{}:{}", record.file(), record.line())),
                );
            }

            json.insert("message".to_string(), json!(record.message()));

            // Add any metadata
            for (key, value) in record.metadata() {
                json.insert(key.to_string(), json!(value));
            }

            // Add any structured data
            for (key, value) in record.context() {
                json.insert(key.to_string(), value.clone());
            }

            format!(
                "{}\n",
                serde_json::to_string(&json).unwrap_or_else(|_| "{}".to_string())
            )
        }
    }

    fn with_colors(self, _use_colors: bool) -> Self {
        self
    }

    fn with_timestamp(mut self, include_timestamp: bool) -> Self {
        self.include_timestamp = include_timestamp;
        self
    }

    fn with_level(mut self, include_level: bool) -> Self {
        self.include_level = include_level;
        self
    }

    fn with_module(mut self, include_module: bool) -> Self {
        self.include_module = include_module;
        self
    }

    fn with_location(mut self, include_location: bool) -> Self {
        self.include_location = include_location;
        self
    }

    fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = pattern.into();
        self
    }

    fn with_format<F>(mut self, format_fn: F) -> Self
    where
        F: Fn(&Record) -> String + Send + Sync + 'static,
    {
        self.format_fn = Some(Arc::new(format_fn));
        self
    }

    fn box_clone(&self) -> Box<dyn FormatterTrait + Send + Sync> {
        Box::new(Self {
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
    fn test_json_formatter_default() {
        let formatter = JsonFormatter::default();
        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        let formatted = formatter.format(&record);
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

        let formatted = formatter.format(&record);
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

        let formatted = formatter.format(&record);
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

        let formatted = formatter.format(&record);
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

        let formatted = formatter.format(&record);
        assert!(formatted.contains("Test message"));
        assert!(formatted.contains("INFO"));
        assert!(formatted.contains("test"));
        assert!(!formatted.contains("test.rs:42"));
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

        let formatted = formatter.format(&record);
        assert_eq!(formatted, "CUSTOM: Test message");
    }
}
