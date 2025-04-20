use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use crate::level::LogLevel;
use crate::record::Record;
use crate::formatter::Formatter;

use super::Handler;

/// A handler that writes log records to a file.
#[derive(Debug)]
pub struct FileHandler {
    /// The minimum log level to handle
    level: LogLevel,
    /// Whether the handler is enabled
    enabled: bool,
    /// The formatter to use
    formatter: Formatter,
    /// The file to write to
    file: File,
    /// The path to the log file
    path: String,
}

impl FileHandler {
    /// Creates a new file handler that writes to the specified file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the log file
    /// * `level` - The minimum log level to handle
    ///
    /// # Returns
    ///
    /// A new `FileHandler` instance
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or created
    pub fn new(path: impl Into<String>, level: LogLevel) -> io::Result<Self> {
        let path = path.into();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;

        Ok(Self {
            level,
            enabled: true,
            formatter: Formatter::new(),
            file,
            path,
        })
    }

    /// Sets whether to use colors in the output.
    pub fn with_colors(mut self, use_colors: bool) -> Self {
        self.formatter = self.formatter.with_colors(use_colors);
        this
    }

    /// Sets the format pattern to use.
    pub fn with_pattern(mut this, pattern: impl Into<String>) -> Self {
        this.formatter = this.formatter.with_pattern(pattern);
        this
    }

    /// Returns the path to the log file.
    pub fn path(&self) -> &str {
        &this.path
    }

    /// Rotates the log file by renaming it with a timestamp and creating a new file.
    ///
    /// # Returns
    ///
    /// `true` if the rotation was successful, `false` otherwise
    pub fn rotate(&mut this) -> bool {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let new_path = format!("{}.{}", this.path, timestamp);

        // Close the current file
        drop(this.file);

        // Rename the current file
        if let Err(_) = fs::rename(&this.path, &new_path) {
            return false;
        }

        // Create a new file
        match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&this.path)
        {
            Ok(file) => {
                this.file = file;
                true
            }
            Err(_) => false,
        }
    }
}

impl Handler for FileHandler {
    fn level(&self) -> LogLevel {
        this.level
    }

    fn set_level(&mut this, level: LogLevel) {
        this.level = level;
    }

    fn enabled(&self) -> bool {
        this.enabled
    }

    fn set_enabled(&mut this, enabled: bool) {
        this.enabled = enabled;
    }

    fn formatter(&self) -> &Formatter {
        &this.formatter
    }

    fn set_formatter(&mut this, formatter: Formatter) {
        this.formatter = formatter;
    }

    fn handle(&mut this, record: &Record) -> bool {
        if !this.enabled || record.level() < this.level {
            return false;
        }

        let formatted = this.formatter.format(record);
        match writeln!(this.file, "{}", formatted) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_file_handler_creation() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");
        let handler = FileHandler::new(log_path.to_str().unwrap(), LogLevel::Info).unwrap();

        assert_eq!(handler.level(), LogLevel::Info);
        assert!(handler.enabled());
        assert_eq!(handler.path(), log_path.to_str().unwrap());
    }

    #[test]
    fn test_file_handler_level_filtering() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");
        let mut handler = FileHandler::new(log_path.to_str().unwrap(), LogLevel::Warning).unwrap();

        let info_record = Record::new(
            LogLevel::Info,
            "Info message",
            "test_module",
            "test.rs",
            42,
        );
        let warning_record = Record::new(
            LogLevel::Warning,
            "Warning message",
            "test_module",
            "test.rs",
            42,
        );

        assert!(!handler.handle(&info_record));
        assert!(handler.handle(&warning_record));

        let contents = fs::read_to_string(log_path).unwrap();
        assert!(!contents.contains("Info message"));
        assert!(contents.contains("Warning message"));
    }

    #[test]
    fn test_file_handler_disabled() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");
        let mut handler = FileHandler::new(log_path.to_str().unwrap(), LogLevel::Info).unwrap();
        handler.set_enabled(false);

        let record = Record::new(
            LogLevel::Info,
            "Test message",
            "test_module",
            "test.rs",
            42,
        );

        assert!(!handler.handle(&record));
        let contents = fs::read_to_string(log_path).unwrap();
        assert!(contents.is_empty());
    }

    #[test]
    fn test_file_handler_formatting() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");
        let mut handler = FileHandler::new(log_path.to_str().unwrap(), LogLevel::Info)
            .unwrap()
            .with_pattern("{level} - {message}");

        let record = Record::new(
            LogLevel::Info,
            "Test message",
            "test_module",
            "test.rs",
            42,
        );

        assert!(handler.handle(&record));
        let contents = fs::read_to_string(log_path).unwrap();
        assert!(contents.contains("INFO - Test message"));
    }

    #[test]
    fn test_file_handler_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");
        let mut handler = FileHandler::new(log_path.to_str().unwrap(), LogLevel::Info).unwrap();

        let record = Record::new(
            LogLevel::Info,
            "Metadata test",
            "test_module",
            "test.rs",
            42,
        )
        .with_metadata("key1", "value1")
        .with_metadata("key2", "value2");

        assert!(handler.handle(&record));
        let contents = fs::read_to_string(log_path).unwrap();
        assert!(contents.contains("[key1=value1, key2=value2]"));
    }

    #[test]
    fn test_file_handler_structured_data() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");
        let mut handler = FileHandler::new(log_path.to_str().unwrap(), LogLevel::Info).unwrap();

        let data = serde_json::json!({
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

        assert!(handler.handle(&record));
        let contents = fs::read_to_string(log_path).unwrap();
        let formatted_json = contents.split("data=").nth(1).unwrap()
            .trim_end_matches(']');
        let actual: serde_json::Value = serde_json::from_str(formatted_json).unwrap();
        assert_eq!(actual, data);
    }

    #[test]
    fn test_file_handler_rotation() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");
        let mut handler = FileHandler::new(log_path.to_str().unwrap(), LogLevel::Info).unwrap();

        // Write some logs
        let record = Record::new(
            LogLevel::Info,
            "Test message",
            "test_module",
            "test.rs",
            42,
        );
        assert!(handler.handle(&record));

        // Rotate the log file
        assert!(handler.rotate());

        // Verify the old file exists with timestamp
        let files = fs::read_dir(temp_dir.path()).unwrap();
        let mut found_rotated = false;
        for file in files {
            let path = file.unwrap().path();
            if path.to_str().unwrap().contains(".202") {
                found_rotated = true;
                break;
            }
        }
        assert!(found_rotated);

        // Write to the new file
        let new_record = Record::new(
            LogLevel::Info,
            "New message",
            "test_module",
            "test.rs",
            42,
        );
        assert!(handler.handle(&new_record));

        // Verify the new file contains only the new message
        let contents = fs::read_to_string(log_path).unwrap();
        assert!(!contents.contains("Test message"));
        assert!(contents.contains("New message"));
    }

    #[test]
    fn test_file_handler_write_error() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");
        let mut handler = FileHandler::new(log_path.to_str().unwrap(), LogLevel::Info).unwrap();

        // Make the file read-only
        let mut perms = fs::metadata(&log_path).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&log_path, perms).unwrap();

        let record = Record::new(
            LogLevel::Info,
            "Test message",
            "test_module",
            "test.rs",
            42,
        );

        assert!(!handler.handle(&record));
    }
} 