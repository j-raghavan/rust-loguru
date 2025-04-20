use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::level::LogLevel;

/// A log record containing all information about a log message
#[derive(Debug, Clone)]
pub struct Record {
    /// The log level
    level: LogLevel,
    /// The log message
    message: String,
    /// The module path
    module: String,
    /// The file name
    file: String,
    /// The line number
    line: u32,
    /// The timestamp
    timestamp: DateTime<Utc>,
    /// Additional metadata
    metadata: HashMap<String, String>,
}

impl Record {
    /// Create a new log record
    pub fn new(
        level: LogLevel,
        message: impl Into<String>,
        module: Option<String>,
        file: Option<String>,
        line: Option<u32>,
    ) -> Self {
        Self {
            level,
            message: message.into(),
            module: module.unwrap_or_else(|| String::from("unknown")),
            file: file.unwrap_or_else(|| String::from("unknown")),
            line: line.unwrap_or(0),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Get the log level
    pub fn level(&self) -> LogLevel {
        self.level
    }

    /// Get the log message
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Get the module path
    pub fn module(&self) -> &str {
        &self.module
    }

    /// Get the file name
    pub fn file(&self) -> &str {
        &self.file
    }

    /// Get the line number
    pub fn line(&self) -> u32 {
        self.line
    }

    /// Get the timestamp
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    /// Get the metadata
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    /// Add metadata to the record
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Adds structured data to the record's metadata.
    ///
    /// The data will be serialized to JSON and stored with the given key.
    /// Returns a Result indicating success or failure of serialization.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rust_loguru::{Record, LogLevel};
    /// use serde_json::json;
    ///
    /// let record = Record::new(LogLevel::Info, "test message", Some("test".to_string()), Some("test.rs".to_string()), Some(42));
    /// let result = record.with_structured_data("user", &json!({
    ///     "id": 123,
    ///     "name": "test"
    /// }));
    /// assert!(result.is_ok());
    /// ```
    pub fn with_structured_data<T: serde::Serialize + ?Sized>(
        mut self,
        key: &str,
        value: &T,
    ) -> Result<Self, serde_json::Error> {
        let json_value = serde_json::to_string(value)?;
        self.metadata.insert(key.to_string(), json_value);
        Ok(self)
    }

    /// Returns the value associated with the given key, if any.
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(String::as_str)
    }
}

impl std::fmt::Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {} {}:{} - {}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
            self.level,
            self.file,
            self.line,
            self.message
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_creation() {
        let record = Record::new(LogLevel::Info, "test message", None, None, None);
        assert_eq!(record.level(), LogLevel::Info);
        assert_eq!(record.message(), "test message");
        assert_eq!(record.module(), "unknown");
        assert_eq!(record.file(), "unknown");
        assert_eq!(record.line(), 0);
    }

    #[test]
    fn test_record_with_metadata() {
        let record = Record::new(LogLevel::Info, "test message", None, None, None)
            .with_metadata("key", "value");
        assert_eq!(record.metadata().get("key").unwrap(), "value");
    }

    #[test]
    fn test_record_with_all_fields() {
        let record = Record::new(
            LogLevel::Info,
            "test message",
            Some("test_module".to_string()),
            Some("test_file.rs".to_string()),
            Some(42),
        );
        assert_eq!(record.module(), "test_module");
        assert_eq!(record.file(), "test_file.rs");
        assert_eq!(record.line(), 42);
    }

    #[test]
    fn test_record_display() {
        let record = Record::new(LogLevel::Error, "Test error message", None, None, None);

        let display = format!("{}", record);
        assert!(display.contains("ERROR"));
        assert!(display.contains("unknown:0"));
        assert!(display.contains("Test error message"));
    }

    #[test]
    fn test_record_metadata_overwrite() {
        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test_module".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        )
        .with_metadata("key", "value1")
        .with_metadata("key", "value2");
        assert_eq!(record.metadata().get("key"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_record_structured_data() {
        let record = Record::new(
            LogLevel::Info,
            "test message",
            Some("test_module".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );
        let record = record.with_structured_data("key", &"value").unwrap();
        assert_eq!(record.metadata().get("key"), Some(&"\"value\"".to_string()));
    }

    #[test]
    fn test_record_timestamp() {
        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test_module".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );
        let now = chrono::Utc::now();
        assert!(record.timestamp() <= now);
    }
}
