use std::fs;
// use std::io;
// use tempfile::tempdir;
// use std::path::Path;
// use std::time::SystemTime;
use std::path::PathBuf;
// use std::time::Duration;
// use std::os::unix::fs::PermissionsExt;

use rust_loguru::formatters::Formatter;
use rust_loguru::handler::file::FileHandler;
use rust_loguru::handler::Handler;
use rust_loguru::level::LogLevel;
use rust_loguru::record::Record;

fn create_test_dir() -> PathBuf {
    let test_dir = PathBuf::from("test_logs");
    if !test_dir.exists() {
        fs::create_dir(&test_dir).unwrap();
    }
    test_dir
}

fn cleanup_test_dir(test_dir: &PathBuf) {
    if test_dir.exists() {
        fs::remove_dir_all(test_dir).unwrap();
    }
}

#[test]
fn test_file_handler_basic() {
    let test_dir = create_test_dir();
    let log_file = test_dir.join("test.log");

    let handler = FileHandler::new(&log_file).expect("Failed to create file handler");
    assert!(handler.is_enabled());
    assert_eq!(handler.level(), LogLevel::Info);

    let record = Record::new(
        LogLevel::Info,
        "test message",
        Some("test".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );

    assert!(handler.handle(&record).is_ok());
    assert!(log_file.exists());

    let contents = fs::read_to_string(&log_file).unwrap();
    assert!(contents.contains("test message"));

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_file_handler_level_filtering() {
    let test_dir = create_test_dir();
    let log_file = test_dir.join("level_test.log");

    let handler = FileHandler::new(&log_file)
        .expect("Failed to create file handler")
        .with_level(LogLevel::Warning)
        .with_colors(false);

    // Info message should not be written
    let info_record = Record::new(
        LogLevel::Info,
        "info message",
        Some("test".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );
    assert!(handler.handle(&info_record).is_ok());

    // Warning message should be written
    let warning_record = Record::new(
        LogLevel::Warning,
        "warning message",
        Some("test".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );
    assert!(handler.handle(&warning_record).is_ok());

    let contents = fs::read_to_string(&log_file).unwrap();
    assert!(!contents.contains("info message"));
    assert!(contents.contains("warning message"));

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_file_handler_enable_disable() {
    let test_dir = create_test_dir();
    let log_file = test_dir.join("enable_test.log");

    let mut handler = FileHandler::new(&log_file).expect("Failed to create file handler");

    // Write while enabled
    let record = Record::new(
        LogLevel::Info,
        "enabled message",
        Some("test".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );
    assert!(handler.handle(&record).is_ok());

    // Disable and write again
    handler.set_enabled(false);
    let record = Record::new(
        LogLevel::Info,
        "disabled message",
        Some("test".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );
    assert!(handler.handle(&record).is_ok());

    let contents = fs::read_to_string(&log_file).unwrap();
    assert!(contents.contains("enabled message"));
    assert!(!contents.contains("disabled message"));

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_file_handler_with_metadata() {
    let test_dir = create_test_dir();
    let log_file = test_dir.join("metadata_test.log");

    let handler = FileHandler::new(&log_file).expect("Failed to create file handler");

    let record = Record::new(
        LogLevel::Info,
        "test message",
        Some("test".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    )
    .with_metadata("key1", "value1")
    .with_metadata("key2", "value2");

    assert!(handler.handle(&record).is_ok());

    let contents = fs::read_to_string(&log_file).unwrap();
    assert!(contents.contains("key1=value1"));
    assert!(contents.contains("key2=value2"));

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_file_handler_formatting() {
    let test_dir = create_test_dir();
    let log_file = test_dir.join("format_test.log");

    let handler = FileHandler::new(&log_file)
        .expect("Failed to create file handler")
        .with_formatter(Formatter::text().with_pattern("{level} - {message}"))
        .with_colors(false);

    let record = Record::new(
        LogLevel::Info,
        "Test message",
        Some("test".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );

    assert!(handler.handle(&record).is_ok());
    let contents = fs::read_to_string(&log_file).unwrap();
    assert!(contents.contains("INFO - Test message"));

    cleanup_test_dir(&test_dir);
}
