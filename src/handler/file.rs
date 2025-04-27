use crate::formatters::Formatter;
use crate::level::LogLevel;
use crate::record::Record;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::sync::Mutex;

use super::{Handler, HandlerFilter};

/// A handler that writes log records to a file
pub struct FileHandler {
    level: LogLevel,
    enabled: bool,
    formatter: Formatter,
    file: Mutex<Option<File>>,
    path: String,
    max_size: Option<usize>,
    max_files: Option<usize>,
    compress: bool,
    filter: Option<HandlerFilter>,
    batch_buffer: Mutex<Vec<Record>>,
    batch_size: Option<usize>,
}

impl fmt::Debug for FileHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileHandler")
            .field("level", &self.level)
            .field("enabled", &self.enabled)
            .field("formatter", &self.formatter)
            .field("path", &self.path)
            .field("max_size", &self.max_size)
            .field("max_files", &self.max_files)
            .field("compress", &self.compress)
            .field("batch_size", &self.batch_size)
            .finish()
    }
}

impl Clone for FileHandler {
    fn clone(&self) -> Self {
        let file = if let Ok(guard) = self.file.lock() {
            if guard.is_some() {
                // Open a new file handle for the clone
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&self.path)
                    .ok()
                    .map(|f| Mutex::new(Some(f)))
                    .unwrap_or_else(|| Mutex::new(None))
            } else {
                Mutex::new(None)
            }
        } else {
            Mutex::new(None)
        };

        Self {
            level: self.level,
            enabled: self.enabled,
            formatter: self.formatter.clone(),
            file,
            path: self.path.clone(),
            max_size: self.max_size,
            max_files: self.max_files,
            compress: self.compress,
            filter: self.filter.clone(),
            batch_buffer: Mutex::new({
                let buffer_guard = self.batch_buffer.lock().unwrap();
                buffer_guard.clone()
            }),
            batch_size: self.batch_size,
        }
    }
}

impl FileHandler {
    pub fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().to_string_lossy().into_owned();
        let file = OpenOptions::new().create(true).append(true).open(&path)?;

        Ok(Self {
            level: LogLevel::Info,
            enabled: true,
            formatter: Formatter::text(),
            file: Mutex::new(Some(file)),
            path,
            max_size: None,
            max_files: None,
            compress: false,
            filter: None,
            batch_buffer: Mutex::new(Vec::new()),
            batch_size: None,
        })
    }

    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }

    pub fn with_formatter(mut self, formatter: Formatter) -> Self {
        self.formatter = formatter;
        self
    }

    pub fn with_colors(mut self, use_colors: bool) -> Self {
        self.formatter = self.formatter.with_colors(use_colors);
        self
    }

    pub fn with_pattern(self, pattern: impl Into<String>) -> Self {
        let mut handler = self;
        let formatter = handler.formatter.with_pattern(pattern);
        handler.formatter = formatter;
        handler
    }

    pub fn with_format<F>(mut self, format_fn: F) -> Self
    where
        F: Fn(&Record) -> String + Send + Sync + 'static,
    {
        self.formatter = self.formatter.with_format(format_fn);
        self
    }

    pub fn with_rotation(mut self, max_size: usize, max_files: usize) -> Self {
        self.max_size = Some(max_size);
        self.max_files = Some(max_files);
        self
    }

    pub fn with_filter(mut self, filter: HandlerFilter) -> Self {
        self.filter = Some(filter);
        self
    }

    pub fn with_compression(mut self, compress: bool) -> Self {
        self.compress = compress;
        self
    }

    pub fn with_batching(mut self, batch_size: usize) -> Self {
        self.batch_size = Some(batch_size);
        self
    }

    fn rotate_if_needed(&self) -> io::Result<()> {
        if let (Some(max_size), Some(max_files)) = (self.max_size, self.max_files) {
            let mut file_guard = self
                .file
                .lock()
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to lock file mutex"))?;

            if let Some(file) = file_guard.as_ref() {
                let metadata = file.metadata()?;
                if metadata.len() as usize >= max_size {
                    // Close the current file
                    *file_guard = None;

                    // Remove the oldest log file if it exists
                    let oldest_log = format!("{}.{}", self.path, max_files);
                    if Path::new(&oldest_log).exists() {
                        std::fs::remove_file(&oldest_log)?;
                    }

                    // Rotate existing files
                    for i in (1..max_files).rev() {
                        let old_path = format!("{}.{}", self.path, i);
                        let new_path = format!("{}.{}", self.path, i + 1);
                        if Path::new(&old_path).exists() {
                            std::fs::rename(&old_path, &new_path)?;
                        }
                    }

                    // Rename current file to .1
                    if Path::new(&self.path).exists() {
                        let rotated_path = format!("{}.1", self.path);
                        std::fs::rename(&self.path, &rotated_path)?;
                        if self.compress {
                            let mut input = File::open(&rotated_path)?;
                            let gz_path = format!("{}.gz", rotated_path);
                            let mut encoder =
                                GzEncoder::new(File::create(&gz_path)?, Compression::default());
                            std::io::copy(&mut input, &mut encoder)?;
                            encoder.finish()?;
                            std::fs::remove_file(&rotated_path)?;
                        }
                    }

                    // Open a new file
                    *file_guard = Some(
                        OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(&self.path)?,
                    );

                    // Flush the new file
                    if let Some(file) = file_guard.as_mut() {
                        file.flush()?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl Handler for FileHandler {
    fn handle(&self, record: &Record) -> Result<(), String> {
        if !self.enabled || record.level() < self.level {
            return Ok(());
        }
        if let Some(filter) = &self.filter {
            if !(filter)(record) {
                return Ok(());
            }
        }
        if let Some(batch_size) = self.batch_size {
            let mut buffer = self.batch_buffer.lock().unwrap();
            buffer.push(record.clone());
            if buffer.len() >= batch_size {
                let batch = buffer.drain(..).collect::<Vec<_>>();
                drop(buffer);
                return self.handle_batch(&batch);
            }
            return Ok(());
        }
        let formatted = self.formatter.format(record);
        if let Err(e) = self.rotate_if_needed() {
            return Err(format!("Failed to rotate log file: {}", e));
        }
        let mut file_guard = self
            .file
            .lock()
            .map_err(|e| format!("Failed to lock file mutex: {}", e))?;
        if let Some(file) = file_guard.as_mut() {
            match write!(file, "{}", formatted) {
                Ok(_) => Ok(()),
                Err(e) => {
                    // Check if it's a permission error
                    if e.kind() == io::ErrorKind::PermissionDenied {
                        Err(format!("Permission denied: {}", e))
                    } else {
                        Err(format!("Failed to write to file: {}", e))
                    }
                }
            }
        } else {
            Err("No file handle available".to_string())
        }
    }

    fn level(&self) -> LogLevel {
        self.level
    }

    fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }

    fn is_enabled(&self) -> bool {
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

    fn set_filter(&mut self, filter: Option<HandlerFilter>) {
        self.filter = filter;
    }

    fn filter(&self) -> Option<&HandlerFilter> {
        self.filter.as_ref()
    }

    fn handle_batch(&self, records: &[Record]) -> Result<(), String> {
        let mut file_guard = self
            .file
            .lock()
            .map_err(|e| format!("Failed to lock file mutex: {}", e))?;
        for record in records {
            if !self.enabled || record.level() < self.level {
                continue;
            }
            if let Some(filter) = &self.filter {
                if !(filter)(record) {
                    continue;
                }
            }
            let formatted = self.formatter.format(record);
            if let Err(e) = self.rotate_if_needed() {
                return Err(format!("Failed to rotate log file: {}", e));
            }
            if let Some(file) = file_guard.as_mut() {
                if let Err(e) = write!(file, "{}", formatted) {
                    return Err(format!("Failed to write to file: {}", e));
                }
            }
        }
        Ok(())
    }

    fn init(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn flush(&self) -> Result<(), String> {
        let mut file_guard = self.file.lock().unwrap();
        if let Some(file) = file_guard.as_mut() {
            file.flush()
                .map_err(|e| format!("Failed to flush file: {}", e))?;
        }
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), String> {
        self.flush()
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
        assert!(handler.is_enabled());
        assert_eq!(handler.path, log_path.to_str().unwrap());
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

        assert!(handler.handle(&info_record).is_ok());
        assert!(handler.handle(&warning_record).is_ok());

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

        assert!(handler.handle(&record).is_ok());
        let contents = fs::read_to_string(log_path)?;
        assert!(contents.is_empty());
        Ok(())
    }

    #[test]
    fn test_file_handler_formatting() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let log_path = temp_dir.path().join("test.log");
        let handler = FileHandler::new(log_path.to_str().unwrap())?
            .with_pattern("{level} - {message}")
            .with_colors(false);

        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test_module".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );

        assert!(handler.handle(&record).is_ok());
        let contents = fs::read_to_string(log_path)?;
        println!("File contents: '{}'", contents);
        println!("File contents length: {}", contents.len());
        println!("File contents bytes: {:?}", contents.as_bytes());

        // Trim whitespace and check again
        let trimmed_contents = contents.trim();
        println!("Trimmed contents: '{}'", trimmed_contents);
        assert!(trimmed_contents.contains("INFO - Test message"));
        Ok(())
    }

    #[test]
    fn test_file_handler_metadata() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let log_path = temp_dir.path().join("test.log");
        let handler = FileHandler::new(log_path.to_str().unwrap())?;

        let record = Record::new(
            LogLevel::Info,
            "Test message",
            Some("test_module".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        )
        .with_metadata("key1", "value1")
        .with_metadata("key2", "value2");

        assert!(handler.handle(&record).is_ok());
        let contents = fs::read_to_string(log_path)?;
        assert!(contents.contains("key1=value1"));
        assert!(contents.contains("key2=value2"));
        Ok(())
    }

    #[test]
    fn test_file_handler_structured_data() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let log_path = temp_dir.path().join("test.log");
        let handler = FileHandler::new(log_path.to_str().unwrap())?;

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

        assert!(handler.handle(&record).is_ok());
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
        let handler = FileHandler::new(log_path.to_str().unwrap())?.with_rotation(100, 3); // Small size to trigger rotation

        // Write enough data to trigger rotation
        let record = Record::new(
            LogLevel::Info,
            "A".repeat(200).as_str(), // Write more than max_size bytes
            Some("test_module".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );
        assert!(handler.handle(&record).is_ok());

        // Write to the new file
        let new_record = Record::new(
            LogLevel::Info,
            "New message",
            Some("test_module".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );
        assert!(handler.handle(&new_record).is_ok());

        // Verify the rotated file exists
        let rotated_path = format!("{}.1", log_path.to_string_lossy());
        assert!(Path::new(&rotated_path).exists());

        // Verify the new file contains only the new message
        let contents = fs::read_to_string(&log_path)?;
        assert!(!contents.contains(&"A".repeat(200)));
        assert!(contents.contains("New message"));

        // Verify the rotated file contains the old message
        let rotated_contents = fs::read_to_string(&rotated_path)?;
        assert!(rotated_contents.contains(&"A".repeat(200)));
        assert!(!rotated_contents.contains("New message"));

        Ok(())
    }

    #[test]
    fn test_file_handler_write_error() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let log_path = temp_dir.path().join("test.log");
        let mut handler = FileHandler::new(log_path.to_str().unwrap())?;

        // Close the file handle before making it read-only
        handler.file = Mutex::new(None);

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

        assert!(handler.handle(&record).is_err());
        Ok(())
    }

    #[test]
    fn test_file_handler_filtering() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let log_path = temp_dir.path().join("test.log");
        let filter = std::sync::Arc::new(|record: &Record| record.message().contains("pass"));
        let handler = FileHandler::new(log_path.to_str().unwrap())?.with_filter(filter);
        let record1 = Record::new(
            LogLevel::Info,
            "should pass",
            None::<String>,
            None::<String>,
            None,
        );
        let record2 = Record::new(
            LogLevel::Info,
            "should fail",
            None::<String>,
            None::<String>,
            None,
        );
        assert!(handler.handle(&record1).is_ok());
        assert!(handler.handle(&record2).is_ok());
        let contents = fs::read_to_string(log_path)?;
        assert!(contents.contains("should pass"));
        assert!(!contents.contains("should fail"));
        Ok(())
    }

    #[test]
    fn test_file_handler_batch() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let log_path = temp_dir.path().join("test.log");
        let handler = FileHandler::new(log_path.to_str().unwrap())?.with_batching(2);
        let record1 = Record::new(LogLevel::Info, "msg1", None::<String>, None::<String>, None);
        let record2 = Record::new(LogLevel::Info, "msg2", None::<String>, None::<String>, None);
        assert!(handler.handle(&record1).is_ok());
        assert!(handler.handle(&record2).is_ok());
        let contents = fs::read_to_string(log_path)?;
        assert!(contents.contains("msg1"));
        assert!(contents.contains("msg2"));
        Ok(())
    }

    #[test]
    fn test_file_handler_compression() -> io::Result<()> {
        let temp_dir = TempDir::new()?;
        let log_path = temp_dir.path().join("test.log");
        let handler = FileHandler::new(log_path.to_str().unwrap())?
            .with_rotation(100, 2)
            .with_compression(true);
        let record1 = Record::new(
            LogLevel::Info,
            "A".repeat(200).as_str(),
            None::<String>,
            None::<String>,
            None,
        );
        let record2 = Record::new(
            LogLevel::Info,
            "B".repeat(200).as_str(),
            None::<String>,
            None::<String>,
            None,
        );
        assert!(handler.handle(&record1).is_ok());
        assert!(handler.handle(&record2).is_ok());
        handler.flush().unwrap();
        let rotated_gz = format!("{}.1.gz", log_path.to_string_lossy());
        assert!(Path::new(&rotated_gz).exists());
        Ok(())
    }
}
