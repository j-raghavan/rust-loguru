use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::HashMap;
use std::fmt;

use crate::level::LogLevel;

/// Represents a single log entry with all its associated metadata.
#[derive(Debug, Clone)]
pub struct Record {
    /// The severity level of the log message.
    level: LogLevel,
    /// The actual log message.
    message: String,
    /// The name of the module that generated the log.
    module: String,
    /// The source file where the log was generated.
    file: String,
    /// The line number in the source file where the log was generated.
    line: u32,
    /// The timestamp when the log was created.
    timestamp: DateTime<Utc>,
    /// Additional key-value pairs associated with the log.
    metadata: HashMap<String, String>,
}

impl Record {
    /// Creates a new log record with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `level` - The severity level of the log message
    /// * `message` - The actual log message
    /// * `module` - The name of the module that generated the log
    /// * `file` - The source file where the log was generated
    /// * `line` - The line number in the source file
    ///
    /// # Returns
    ///
    /// A new `Record` instance with the current timestamp and empty metadata.
    pub fn new(
        level: LogLevel,
        message: impl Into<String>,
        module: impl Into<String>,
        file: impl Into<String>,
        line: u32,
    ) -> Self {
        Self {
            level,
            message: message.into(),
            module: module.into(),
            file: file.into(),
            line,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Adds a key-value pair to the record's metadata.
    ///
    /// # Arguments
    ///
    /// * `key` - The metadata key
    /// * `value` - The metadata value
    ///
    /// # Returns
    ///
    /// The modified `Record` instance for method chaining.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Adds structured data to the record's metadata.
    ///
    /// # Arguments
    ///
    /// * `key` - The metadata key
    /// * `data` - The structured data to serialize
    ///
    /// # Returns
    ///
    /// The modified `Record` instance for method chaining.
    ///
    /// # Errors
    ///
    /// Returns an error if the data cannot be serialized to JSON.
    pub fn with_structured_data<T: Serialize>(
        mut self,
        key: impl Into<String>,
        data: &T,
    ) -> Result<Self, serde_json::Error> {
        let json = serde_json::to_string(data)?;
        self.metadata.insert(key.into(), json);
        Ok(self)
    }

    /// Returns the log level of this record.
    pub fn level(&self) -> LogLevel {
        self.level
    }

    /// Returns the log message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the module name.
    pub fn module(&self) -> &str {
        &self.module
    }

    /// Returns the source file path.
    pub fn file(&self) -> &str {
        &self.file
    }

    /// Returns the line number.
    pub fn line(&self) -> u32 {
        self.line
    }

    /// Returns the timestamp when the log was created.
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    /// Returns a reference to the metadata map.
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    /// Returns the value associated with the given key, if any.
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(String::as_str)
    }
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
    use serde_json::json;

    #[test]
    fn test_record_creation() {
        let record = Record::new(LogLevel::Info, "Test message", "test_module", "test.rs", 42);

        assert_eq!(record.level(), LogLevel::Info);
        assert_eq!(record.message(), "Test message");
        assert_eq!(record.module(), "test_module");
        assert_eq!(record.file(), "test.rs");
        assert_eq!(record.line(), 42);
    }

    #[test]
    fn test_record_metadata() {
        let record = Record::new(LogLevel::Info, "Test message", "test_module", "test.rs", 42)
            .with_metadata("key1", "value1")
            .with_metadata("key2", "value2");

        assert_eq!(record.get_metadata("key1"), Some("value1"));
        assert_eq!(record.get_metadata("key2"), Some("value2"));
        assert_eq!(record.get_metadata("nonexistent"), None);
    }

    #[test]
    fn test_record_structured_data() {
        let data = json!({
            "user_id": 123,
            "action": "login",
            "success": true
        });

        let record = Record::new(LogLevel::Info, "User action", "test_module", "test.rs", 42)
            .with_structured_data("user_data", &data)
            .unwrap();

        let stored_data = record.get_metadata("user_data").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(stored_data).unwrap();
        assert_eq!(parsed["user_id"], 123);
        assert_eq!(parsed["action"], "login");
        assert_eq!(parsed["success"], true);
    }

    #[test]
    fn test_record_display() {
        let record = Record::new(
            LogLevel::Error,
            "Test error message",
            "test_module",
            "test.rs",
            42,
        );

        let display = format!("{}", record);
        assert!(display.contains("ERROR"));
        assert!(display.contains("test.rs:42"));
        assert!(display.contains("Test error message"));
    }
}
