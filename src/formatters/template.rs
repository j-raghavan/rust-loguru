use colored::*;
use std::fmt;
use std::sync::Arc;

use crate::formatters::{FormatFn, FormatterTrait};
use crate::level::LogLevel;
use crate::record::Record;

/// A template formatter that formats log records using a template
#[derive(Clone)]
pub struct TemplateFormatter {
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
    /// Whether to include metadata in the output
    include_metadata: bool,
    /// Whether to include structured data in the output
    include_data: bool,
    /// The format pattern to use
    pattern: String,
    /// A custom format function
    format_fn: Option<FormatFn>,
    /// Pre-computed placeholder positions for faster formatting
    placeholders: Vec<(usize, usize, String)>,
}

impl fmt::Debug for TemplateFormatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TemplateFormatter")
            .field("use_colors", &self.use_colors)
            .field("include_timestamp", &self.include_timestamp)
            .field("include_level", &self.include_level)
            .field("include_module", &self.include_module)
            .field("include_location", &self.include_location)
            .field("include_metadata", &self.include_metadata)
            .field("include_data", &self.include_data)
            .field("pattern", &self.pattern)
            .field("format_fn", &"<format_fn>")
            .finish()
    }
}

impl Default for TemplateFormatter {
    fn default() -> Self {
        let pattern =
            "{timestamp} {level} {module} {location} {message} {metadata} {data}".to_string();
        let mut formatter = Self {
            use_colors: true,
            include_timestamp: true,
            include_level: true,
            include_module: true,
            include_location: true,
            include_metadata: true,
            include_data: true,
            pattern: pattern.clone(),
            format_fn: None,
            placeholders: Vec::new(),
        };
        formatter.update_placeholders();
        formatter
    }
}

impl TemplateFormatter {
    pub fn new(pattern: impl Into<String>) -> Self {
        let pattern = pattern.into();
        let mut formatter = Self {
            use_colors: true,
            include_timestamp: pattern.contains("{timestamp}"),
            include_level: pattern.contains("{level}"),
            include_module: pattern.contains("{module}"),
            include_location: pattern.contains("{location}"),
            include_metadata: pattern.contains("{metadata}"),
            include_data: pattern.contains("{data}"),
            pattern,
            format_fn: None,
            placeholders: Vec::new(),
        };
        formatter.update_placeholders();
        formatter
    }

    fn update_flags_from_pattern(&mut self) {
        self.include_timestamp = self.pattern.contains("{timestamp}");
        self.include_level = self.pattern.contains("{level}");
        self.include_module = self.pattern.contains("{module}");
        self.include_location = self.pattern.contains("{location}");
        self.include_metadata = self.pattern.contains("{metadata}");
        self.include_data = self.pattern.contains("{data}");
    }

    fn update_placeholders(&mut self) {
        self.placeholders.clear();
        let mut pos = 0;
        while let Some(start) = self.pattern[pos..].find('{') {
            if let Some(end) = self.pattern[pos + start..].find('}') {
                let placeholder = self.pattern[pos + start + 1..pos + start + end].to_string();
                self.placeholders
                    .push((pos + start, pos + start + end + 1, placeholder));
                pos += start + end + 1;
            } else {
                break;
            }
        }
    }

    pub fn with_colors(mut self, use_colors: bool) -> Self {
        self.use_colors = use_colors;
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

    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = pattern.into();
        self.update_flags_from_pattern();
        self.update_placeholders();
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

impl FormatterTrait for TemplateFormatter {
    fn fmt(&self, record: &Record) -> String {
        if let Some(format_fn) = &self.format_fn {
            // Return custom format as-is without modifications
            let result = format_fn(record);
            if result.ends_with('\n') {
                result[..result.len() - 1].to_string()
            } else {
                result
            }
        } else {
            // Pre-allocate with estimated capacity
            let mut result = String::with_capacity(self.pattern.len() * 2);
            let mut last_pos = 0;

            // Use pre-computed placeholder positions for single-pass replacement
            for &(start, end, ref placeholder) in &self.placeholders {
                result.push_str(&self.pattern[last_pos..start]);

                match placeholder.as_str() {
                    "timestamp" => {
                        if self.include_timestamp {
                            result.push_str(&record.timestamp().to_rfc3339())
                        }
                    }
                    "level" => {
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
                        }
                    }
                    "module" => {
                        if self.include_module {
                            let module = record.module();
                            if module != "unknown" {
                                result.push_str(module);
                            }
                        }
                    }
                    "location" => {
                        if self.include_location {
                            // Only include file and line, not module
                            let file = record.file();
                            let line = record.line();
                            if file != "unknown" {
                                result.push_str(&format!("{}:{}", file, line));
                            }
                        }
                    }
                    "message" => {
                        result.push_str(record.message());
                    }
                    "metadata" => {
                        if self.include_metadata {
                            let metadata = record.metadata();
                            if !metadata.is_empty() {
                                let metadata_str = metadata
                                    .iter()
                                    .map(|(k, v)| format!("{}={}", k, v))
                                    .collect::<Vec<_>>()
                                    .join(" ");
                                result.push_str(&metadata_str);
                            }
                        }
                    }
                    "data" => {
                        if self.include_data {
                            let metadata = record.metadata();
                            if !metadata.is_empty() {
                                let data_str = metadata
                                    .iter()
                                    .filter(|(k, _)| k.starts_with("data."))
                                    .map(|(k, v)| format!("{}={}", k, v))
                                    .collect::<Vec<_>>()
                                    .join(" ");
                                if !data_str.is_empty() {
                                    result.push_str(&data_str);
                                }
                            }
                        }
                    }
                    _ => {
                        result.push_str(&self.pattern[start..end]);
                    }
                }
                last_pos = end;
            }

            // Add remaining pattern content
            result.push_str(&self.pattern[last_pos..]);

            // Only add newline for default formatting
            if !result.ends_with('\n') {
                result.push('\n');
            }

            result
        }
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
        self.update_flags_from_pattern();
        self.update_placeholders();
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

    #[test]
    fn test_template_formatter_default() {
        let formatter = TemplateFormatter::default();
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
    fn test_template_formatter_no_colors() {
        let formatter = TemplateFormatter::default().with_colors(false);
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
        assert!(!formatted.contains("\x1b["));
    }

    #[test]
    fn test_template_formatter_no_timestamp() {
        let formatter = TemplateFormatter::default().with_timestamp(false);
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
        assert!(!formatted.contains("2023")); // No year in timestamp
    }

    #[test]
    fn test_template_formatter_no_level() {
        let formatter = TemplateFormatter::default().with_level(false);
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
    fn test_template_formatter_no_module() {
        let formatter = TemplateFormatter::default().with_module(false);
        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test_module".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        let formatted = formatter.fmt(&record);
        assert!(formatted.contains("Test message"));
        assert!(formatted.contains("INFO"));
        assert!(!formatted.contains("test_module"));
        assert!(formatted.contains("test.rs:42"));
    }

    #[test]
    fn test_template_formatter_no_location() {
        let formatter = TemplateFormatter::default().with_location(false);
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
        assert!(!formatted.contains("test.rs:42"));
    }

    #[test]
    fn test_template_formatter_custom_format() {
        let formatter = TemplateFormatter::default()
            .with_format(|record: &Record| format!("CUSTOM: {}", record.message()));
        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test_module".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        let formatted = formatter.fmt(&record);
        assert_eq!(formatted, "CUSTOM: Test message");
    }
}
