use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::formatter::Formatter;
use crate::level::LogLevel;
use crate::record::Record;

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
    /// The file path
    path: PathBuf,
    /// The current file handle
    file: Option<File>,
    /// The maximum file size before rotation
    max_size: Option<u64>,
    /// The number of old files to keep
    retention: Option<usize>,
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
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = OpenOptions::new().create(true).append(true).open(&path)?;

        Ok(Self {
            level: LogLevel::Info,
            enabled: true,
            formatter: Formatter::new(),
            path,
            file: Some(file),
            max_size: None,
            retention: None,
        })
    }

    /// Closes the file handle if it's open.
    pub fn close(&mut self) {
        self.file = None;
    }

    /// Sets whether to use colors in the output.
    pub fn with_colors(mut self, use_colors: bool) -> Self {
        self.formatter = self.formatter.with_colors(use_colors);
        self
    }

    /// Sets the format pattern to use.
    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.formatter = self.formatter.with_pattern(pattern);
        self
    }

    /// Returns the path to the log file.
    pub fn path(&self) -> &str {
        self.path.to_str().unwrap()
    }

    /// Sets the maximum file size before rotation
    pub fn with_rotation(mut self, max_size: u64) -> Self {
        self.max_size = Some(max_size);
        self
    }

    /// Sets the number of old files to keep
    pub fn with_retention(mut self, retention: usize) -> Self {
        self.retention = Some(retention);
        self
    }

    /// Rotate the log file if necessary
    fn rotate_if_needed(&mut self) -> io::Result<()> {
        if let Some(max_size) = self.max_size {
            // Ensure the file is flushed before checking size
            if let Some(file) = &mut self.file {
                file.flush()?;
            }

            let metadata = fs::metadata(&self.path)?;
            if metadata.len() >= max_size {
                self.rotate()?;
            }
        }
        Ok(())
    }

    /// Rotate the log file
    fn rotate(&mut self) -> io::Result<()> {
        // Close the current file
        self.file = None;

        // Rotate old files first
        if let Some(retention) = self.retention {
            for i in (1..retention).rev() {
                let old_path = self.path.with_extension(format!("log.{}", i));
                let new_path = self.path.with_extension(format!("log.{}", i + 1));
                if old_path.exists() {
                    fs::rename(&old_path, &new_path)?;
                }
            }
        }

        // Rename the current file
        let rotated_path = self.path.with_extension("log.1");
        if self.path.exists() {
            fs::rename(&self.path, &rotated_path)?;
        }

        // Create a new file
        self.file = Some(
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&self.path)?,
        );

        Ok(())
    }
}

impl Handler for FileHandler {
    fn level(&self) -> LogLevel {
        self.level
    }

    fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn formatter(&self) -> &Formatter {
        &self.formatter
    }

    fn set_formatter(&mut self, formatter: Formatter) {
        self.formatter = formatter;
    }

    fn handle(&mut self, record: &Record) -> bool {
        if !self.enabled || record.level() < self.level {
            return false;
        }

        // Format the record first to avoid unnecessary file operations
        let formatted = self.formatter.format(record);

        // If we don't have a file handle, try to open one
        if self.file.is_none() {
            match OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)
            {
                Ok(file) => self.file = Some(file),
                Err(_) => return false,
            }
        }

        // Try to write with the file handle
        if let Some(file) = &mut self.file {
            if writeln!(file, "{}", formatted).is_err() {
                self.file = None;
                return false;
            }
            if file.flush().is_err() {
                self.file = None;
                return false;
            }
        } else {
            return false;
        }

        // Rotate if needed
        if self.rotate_if_needed().is_err() {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
  
    fn test_file_handler_creation() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let log_path = temp_dir.path().join("test.log");
        let handler = FileHandler::new(log_path.to_str().unwrap())?;

        assert_eq!(handler.level(), LogLevel::Info);
        assert!(handler.enabled());
        assert_eq!(handler.path(), log_path.to_str().unwrap());
        Ok(())
    }

    #[test]
    fn test_file_handler_level_filtering() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let log_path = temp_dir.path().join("test.log");
        let mut handler = FileHandler::new(log_path.to_str().unwrap())?;
        handler.set_level(LogLevel::Warning);

        let info_record = Record::new(
            LogLevel::Info,
            "Info message",
            Some("test_module".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );
        let warning_record = Record::new(
            LogLevel::Warning,
            "Warning message",
            Some("test_module".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        assert!(!handler.handle(&info_record));
        assert!(handler.handle(&warning_record));

        let contents = fs::read_to_string(log_path)?;
        assert!(!contents.contains("Info message"));
        assert!(contents.contains("Warning message"));
        Ok(())
    }

    #[test]
    fn test_file_handler_disabled() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let log_path = temp_dir.path().join("test.log");
        let mut handler = FileHandler::new(log_path.to_str().unwrap())?;
        handler.set_enabled(false);

        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test_module".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        assert!(!handler.handle(&record));
        let contents = fs::read_to_string(log_path)?;
        assert!(contents.is_empty());
        Ok(())
    }

    #[test]
    fn test_file_handler_formatting() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let log_path = temp_dir.path().join("test.log");
        let mut handler =
            FileHandler::new(log_path.to_str().unwrap())?.with_pattern("{level} - {message}");

        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test_module".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        assert!(handler.handle(&record));
        let contents = fs::read_to_string(log_path)?;
        assert!(contents.contains("INFO - Test message"));
        Ok(())
    }

    #[test]
    fn test_file_handler_metadata() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let log_path = temp_dir.path().join("test.log");
        let mut handler = FileHandler::new(log_path.to_str().unwrap())?;

        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test_module".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        )
        .with_metadata("key1", "value1")
        .with_metadata("key2", "value2");

        assert!(handler.handle(&record));
        let contents = fs::read_to_string(log_path)?;
        assert!(contents.contains("key1=value1"));
        assert!(contents.contains("key2=value2"));
        Ok(())
    }

    #[test]
    fn test_file_handler_structured_data() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let log_path = temp_dir.path().join("test.log");
        let mut handler = FileHandler::new(log_path.to_str().unwrap())?;

        let data = serde_json::json!({
            "user_id": 123,
            "action": "login"
        });

        let record = Record::new(
            LogLevel::Info,
            "Structured data test",
            Some("test_module".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        )
        .with_structured_data("data", &data)
        .unwrap();

        assert!(handler.handle(&record));
        let contents = fs::read_to_string(log_path)?;
        assert!(contents.contains("data="));
        assert!(contents.contains(r#""user_id":123"#));
        assert!(contents.contains(r#""action":"login""#));
        Ok(())
    }

    #[test]
    fn test_file_handler_rotation() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let log_path = temp_dir.path().join("test.log");
        let mut handler = FileHandler::new(log_path.to_str().unwrap())?.with_rotation(1000); // 1KB rotation size

        // Write some logs
        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test_module".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );
        assert!(handler.handle(&record));

        // Force a rotation
        handler.rotate()?;

        // Verify the rotated file exists
        let rotated_path = log_path.with_extension("log.1");
        assert!(rotated_path.exists());

        // Write to the new file
        let new_record = Record::new(
            LogLevel::Info,
            "New message",
            Some("test_module".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );
        assert!(handler.handle(&new_record));

        // Verify the new file contains only the new message
        let contents = fs::read_to_string(&log_path)?;
        assert!(!contents.contains("Test message"));
        assert!(contents.contains("New message"));

        // Verify the rotated file contains the old message
        let rotated_contents = fs::read_to_string(&rotated_path)?;
        assert!(rotated_contents.contains("Test message"));
        assert!(!rotated_contents.contains("New message"));

        Ok(())
    }

    #[test]
    fn test_file_handler_write_error() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let log_path = temp_dir.path().join("test.log");
        let mut handler = FileHandler::new(log_path.to_str().unwrap())?;

        // Close the file handle before making it read-only
        handler.file = None;

        // Make the file read-only to simulate a write error
        let mut perms = fs::metadata(&log_path)?.permissions();
        perms.set_readonly(true);
        fs::set_permissions(&log_path, perms)?;

        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test_module".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        assert!(!handler.handle(&record));
        Ok(())
    }
}
