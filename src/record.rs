use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::level::LogLevel;

/// Type alias for the record formatting function.
/// This function takes a reference to a Record and returns a formatted String.
type RecordFormatter = Box<dyn Fn(&Record) -> String + Send + Sync>;

/// A log record containing all information about a log message
///
/// This struct is designed to be thread-safe and can be safely shared between threads.
/// All methods that modify the record take ownership and return a new instance,
/// ensuring thread safety without the need for locks.
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
    /// Structured context data
    context: HashMap<String, serde_json::Value>,
    /// Deferred formatting function
    format_fn: Option<RecordFormatter>,
}

impl Clone for Record {
    fn clone(&self) -> Self {
        Self {
            level: self.level,
            message: self.message.clone(),
            module: self.module.clone(),
            file: self.file.clone(),
            line: self.line,
            timestamp: self.timestamp,
            metadata: self.metadata.clone(),
            context: self.context.clone(),
            format_fn: None, // We don't clone the format function
        }
    }
}

impl fmt::Debug for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Record")
            .field("level", &self.level)
            .field("message", &self.message)
            .field("module", &self.module)
            .field("file", &self.file)
            .field("line", &self.line)
            .field("timestamp", &self.timestamp)
            .field("metadata", &self.metadata)
            .field("context", &self.context)
            .field("format_fn", &format_args!("<function>"))
            .finish()
    }
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
            context: HashMap::new(),
            format_fn: None,
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

    /// Get the context data
    pub fn context(&self) -> &HashMap<String, serde_json::Value> {
        &self.context
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

    /// Adds structured context data to the record.
    ///
    /// The data will be stored as a serde_json::Value and can be used for
    /// structured logging and analysis.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rust_loguru::{Record, LogLevel};
    /// use serde_json::json;
    ///
    /// let record = Record::new(LogLevel::Info, "test message", None, None, None);
    /// let record = record.with_context("user", json!({
    ///     "id": 123,
    ///     "name": "test"
    /// }));
    /// ```
    pub fn with_context(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.context.insert(key.into(), value);
        self
    }

    /// Sets a deferred formatting function for the record.
    ///
    /// This allows for lazy evaluation of the record's string representation,
    /// which can improve performance when the record is not actually displayed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rust_loguru::{Record, LogLevel};
    ///
    /// let record = Record::new(LogLevel::Info, "test message", None, None, None);
    /// let record = record.with_deferred_format(|r| {
    ///     format!("[{}] {} - {}", r.timestamp(), r.level(), r.message())
    /// });
    /// ```
    pub fn with_deferred_format<F>(mut self, format_fn: F) -> Self
    where
        F: Fn(&Record) -> String + Send + Sync + 'static,
    {
        self.format_fn = Some(Box::new(format_fn));
        self
    }

    /// Returns the value associated with the given key, if any.
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(String::as_str)
    }

    /// Returns the context value associated with the given key, if any.
    pub fn get_context(&self, key: &str) -> Option<&serde_json::Value> {
        self.context.get(key)
    }

    /// Returns true if the record has any structured context data.
    pub fn has_context(&self) -> bool {
        !self.context.is_empty()
    }

    /// Returns true if the record has any metadata.
    pub fn has_metadata(&self) -> bool {
        !self.metadata.is_empty()
    }

    /// Returns true if the record has a deferred formatter.
    pub fn has_formatter(&self) -> bool {
        self.format_fn.is_some()
    }

    /// Returns the number of context entries in the record.
    pub fn context_len(&self) -> usize {
        self.context.len()
    }

    /// Returns the number of metadata entries in the record.
    pub fn metadata_len(&self) -> usize {
        self.metadata.len()
    }
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(format_fn) = &self.format_fn {
            write!(f, "{}", format_fn(self))
        } else {
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
}

impl Serialize for Record {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Record", 7)?;
        state.serialize_field("level", &self.level)?;
        state.serialize_field("message", &self.message)?;
        state.serialize_field("module", &self.module)?;
        state.serialize_field("file", &self.file)?;
        state.serialize_field("line", &self.line)?;
        state.serialize_field("timestamp", &self.timestamp.to_rfc3339())?;
        state.serialize_field("metadata", &self.metadata)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Record {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Level,
            Message,
            Module,
            File,
            Line,
            Timestamp,
            Metadata,
        }

        struct RecordVisitor;

        impl<'de> serde::de::Visitor<'de> for RecordVisitor {
            type Value = Record;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Record")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Record, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut level = None;
                let mut message = None;
                let mut module = None;
                let mut file = None;
                let mut line = None;
                let mut timestamp = None;
                let mut metadata = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Level => {
                            if level.is_some() {
                                return Err(serde::de::Error::duplicate_field("level"));
                            }
                            level = Some(map.next_value()?);
                        }
                        Field::Message => {
                            if message.is_some() {
                                return Err(serde::de::Error::duplicate_field("message"));
                            }
                            message = Some(map.next_value()?);
                        }
                        Field::Module => {
                            if module.is_some() {
                                return Err(serde::de::Error::duplicate_field("module"));
                            }
                            module = Some(map.next_value()?);
                        }
                        Field::File => {
                            if file.is_some() {
                                return Err(serde::de::Error::duplicate_field("file"));
                            }
                            file = Some(map.next_value()?);
                        }
                        Field::Line => {
                            if line.is_some() {
                                return Err(serde::de::Error::duplicate_field("line"));
                            }
                            line = Some(map.next_value()?);
                        }
                        Field::Timestamp => {
                            if timestamp.is_some() {
                                return Err(serde::de::Error::duplicate_field("timestamp"));
                            }
                            let ts_str: String = map.next_value()?;
                            timestamp = Some(
                                DateTime::parse_from_rfc3339(&ts_str)
                                    .map_err(serde::de::Error::custom)?
                                    .with_timezone(&Utc),
                            );
                        }
                        Field::Metadata => {
                            if metadata.is_some() {
                                return Err(serde::de::Error::duplicate_field("metadata"));
                            }
                            metadata = Some(map.next_value()?);
                        }
                    }
                }

                let level = level.ok_or_else(|| serde::de::Error::missing_field("level"))?;
                let message = message.ok_or_else(|| serde::de::Error::missing_field("message"))?;
                let module = module.ok_or_else(|| serde::de::Error::missing_field("module"))?;
                let file = file.ok_or_else(|| serde::de::Error::missing_field("file"))?;
                let line = line.ok_or_else(|| serde::de::Error::missing_field("line"))?;
                let timestamp =
                    timestamp.ok_or_else(|| serde::de::Error::missing_field("timestamp"))?;
                let metadata = metadata.unwrap_or_default();

                Ok(Record {
                    level,
                    message,
                    module,
                    file,
                    line,
                    timestamp,
                    metadata,
                    context: HashMap::new(),
                    format_fn: None,
                })
            }
        }

        deserializer.deserialize_struct(
            "Record",
            &[
                "level",
                "message",
                "module",
                "file",
                "line",
                "timestamp",
                "metadata",
            ],
            RecordVisitor,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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

    #[test]
    fn test_record_context() {
        let record = Record::new(LogLevel::Info, "test message", None, None, None).with_context(
            "user",
            json!({
                "id": 123,
                "name": "test"
            }),
        );

        let user = record.get_context("user").unwrap();
        assert_eq!(user["id"], 123);
        assert_eq!(user["name"], "test");
    }

    #[test]
    fn test_record_deferred_format() {
        let record = Record::new(LogLevel::Info, "test message", None, None, None)
            .with_deferred_format(|r| {
                format!("[{}] {} - {}", r.timestamp(), r.level(), r.message())
            });

        let display = format!("{}", record);
        assert!(display.contains("INFO"));
        assert!(display.contains("test message"));
    }

    #[test]
    fn test_record_serialization() {
        let record = Record::new(
            LogLevel::Info,
            "test message",
            Some("test_module".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        let serialized = serde_json::to_string(&record).unwrap();
        let deserialized: Record = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.level(), record.level());
        assert_eq!(deserialized.message(), record.message());
        assert_eq!(deserialized.module(), record.module());
        assert_eq!(deserialized.file(), record.file());
        assert_eq!(deserialized.line(), record.line());
    }

    #[test]
    fn test_record_state_checks() {
        let record = Record::new(LogLevel::Info, "test message", None, None, None);
        assert!(!record.has_context());
        assert!(!record.has_metadata());
        assert!(!record.has_formatter());
        assert_eq!(record.context_len(), 0);
        assert_eq!(record.metadata_len(), 0);

        let record = record
            .with_metadata("key", "value")
            .with_context("user", json!({"id": 1}))
            .with_deferred_format(|r| format!("{}", r.message()));

        assert!(record.has_context());
        assert!(record.has_metadata());
        assert!(record.has_formatter());
        assert_eq!(record.context_len(), 1);
        assert_eq!(record.metadata_len(), 1);
    }
}
