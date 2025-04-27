use crate::formatters::FormatterTrait;
use crate::record::Record;
use chrono::Local;
use serde_json;
use std::fmt;
use std::sync::Arc;

/// A type alias for a format function
pub type FormatFn = Arc<dyn Fn(&Record) -> String + Send + Sync>;

/// Text formatter implementation
#[derive(Clone)]
pub struct TextFormatter {
    use_colors: bool,
    include_timestamp: bool,
    include_level: bool,
    include_module: bool,
    include_location: bool,
    pattern: String,
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

impl TextFormatter {
    pub fn new() -> Self {
        Self::default()
    }
}

impl FormatterTrait for TextFormatter {
    fn format(&self, record: &Record) -> String {
        // If a custom format function is provided, use it
        if let Some(format_fn) = &self.format_fn {
            return format_fn(record);
        }

        let mut result = self.pattern.clone();

        // Helper closure to replace placeholders only if value exists
        let replace_if = |text: &mut String, placeholder: &str, value: Option<&str>| {
            if let Some(val) = value {
                if !val.is_empty() {
                    *text = text.replace(placeholder, val);
                } else {
                    // Remove the placeholder and any surrounding whitespace
                    *text = text
                        .replace(&format!(" {}", placeholder), "")
                        .replace(&format!("{} ", placeholder), "")
                        .replace(placeholder, "");
                }
            } else {
                // Remove the placeholder and any surrounding whitespace
                *text = text
                    .replace(&format!(" {}", placeholder), "")
                    .replace(&format!("{} ", placeholder), "")
                    .replace(placeholder, "");
            }
        };

        // Replace message first
        replace_if(&mut result, "{message}", Some(record.message()));

        // Replace level if included
        if self.include_level {
            let level_str = if self.use_colors {
                record.level().to_string_colored()
            } else {
                record.level().to_string()
            };
            replace_if(&mut result, "{level}", Some(&level_str));
        } else {
            replace_if(&mut result, "{level}", None);
        }

        // Replace module if included
        if self.include_module {
            replace_if(&mut result, "{module}", Some(record.module()));
        } else {
            replace_if(&mut result, "{module}", None);
        }

        // Replace location if included
        if self.include_location {
            let location = if !record.file().is_empty() {
                Some(format!("{}:{}", record.file(), record.line()))
            } else {
                None
            };
            replace_if(&mut result, "{location}", location.as_deref());
        } else {
            replace_if(&mut result, "{location}", None);
        }

        // Replace timestamp if included
        if self.include_timestamp {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
            replace_if(&mut result, "{timestamp}", Some(&timestamp));
        } else {
            replace_if(&mut result, "{timestamp}", None);
        }

        // Add metadata to the output
        if !record.metadata().is_empty() {
            let metadata_str = record
                .metadata()
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(" ");
            if !metadata_str.is_empty() {
                result = format!("{} {}", result.trim_end(), metadata_str);
            }
        }

        // Add structured data to the output
        if !record.context().is_empty() {
            let context_str = record
                .context()
                .iter()
                .map(|(k, v)| format!("{}={}", k, serde_json::to_string(v).unwrap_or_default()))
                .collect::<Vec<_>>()
                .join(" ");
            if !context_str.is_empty() {
                result = format!("{} {}", result.trim_end(), context_str);
            }
        }

        // Clean up whitespace while preserving newlines and indentation
        result = result
            .lines()
            .map(|line| {
                let trimmed = line.trim_end();
                if trimmed.is_empty() {
                    String::new()
                } else {
                    trimmed.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Ensure newline at end
        if !result.ends_with('\n') {
            result.push('\n');
        }

        result
    }

    fn with_colors(mut self, use_colors: bool) -> Self {
        self.use_colors = use_colors;
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
        // Update the internal flags based on the pattern content
        self.include_timestamp = self.pattern.contains("{timestamp}");
        self.include_level = self.pattern.contains("{level}");
        self.include_module = self.pattern.contains("{module}");
        self.include_location = self.pattern.contains("{location}");
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

        let formatted = formatter.format(&record);
        assert!(formatted.contains("Test message"));
        assert!(formatted.contains("INFO"));
        assert!(formatted.contains("test"));
        assert!(formatted.contains("test.rs:42"));
    }

    #[test]
    fn test_text_formatter_no_colors() {
        let formatter = TextFormatter::default().with_colors(false);
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
        assert!(!formatted.contains("\x1b["));
    }

    #[test]
    fn test_text_formatter_no_timestamp() {
        let formatter = TextFormatter::default().with_timestamp(false);
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
        assert!(!formatted.contains("2023")); // No year in timestamp
    }

    #[test]
    fn test_text_formatter_no_level() {
        let formatter = TextFormatter::default().with_level(false);
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
    fn test_text_formatter_no_module() {
        let formatter = TextFormatter::default().with_module(false);
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
    fn test_text_formatter_no_location() {
        let formatter = TextFormatter::default().with_location(false);
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
    fn test_text_formatter_custom_format() {
        let formatter =
            TextFormatter::default().with_format(|record| format!("CUSTOM: {}", record.message()));
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
