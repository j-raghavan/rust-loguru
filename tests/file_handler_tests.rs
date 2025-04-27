use std::fs;
use std::path::PathBuf;
use std::sync::Once;

use rust_loguru::formatters::Formatter;
use rust_loguru::handler::file::FileHandler;
use rust_loguru::handler::Handler;
use rust_loguru::level::LogLevel;
use rust_loguru::record::Record;

// Used to ensure directory is created only once during test suite run
static INIT: Once = Once::new();

fn setup_test_dir() -> PathBuf {
    let test_dir = PathBuf::from("test_logs");

    INIT.call_once(|| {
        // Clean up any existing directory from previous test runs
        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).unwrap_or_else(|e| {
                eprintln!("Warning: Could not remove existing test directory: {}", e);
            });
        }

        // Create a fresh directory
        fs::create_dir_all(&test_dir).unwrap_or_else(|e| {
            panic!("Failed to create test directory: {}", e);
        });
    });

    test_dir.clone()
}

fn cleanup_log_file(log_file: &PathBuf) {
    if log_file.exists() {
        fs::remove_file(log_file).unwrap_or_else(|e| {
            eprintln!("Warning: Could not remove log file: {}", e);
        });
    }
}

#[test]
fn test_file_handler_basic() {
    let test_dir = setup_test_dir();
    let log_file = test_dir.join("test.log");

    // Ensure clean state
    cleanup_log_file(&log_file);

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

    // Clean up this test's file
    cleanup_log_file(&log_file);
}

#[test]
fn test_file_handler_level_filtering() {
    let test_dir = setup_test_dir();
    let log_file = test_dir.join("level_test.log");

    // Ensure clean state
    cleanup_log_file(&log_file);

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

    // Clean up this test's file
    cleanup_log_file(&log_file);
}

#[test]
fn test_file_handler_enable_disable() {
    let test_dir = setup_test_dir();
    let log_file = test_dir.join("enable_test.log");

    // Ensure clean state
    cleanup_log_file(&log_file);

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

    // Clean up this test's file
    cleanup_log_file(&log_file);
}

#[test]
fn test_file_handler_with_metadata() {
    let test_dir = setup_test_dir();
    let log_file = test_dir.join("metadata_test.log");

    // Ensure clean state
    cleanup_log_file(&log_file);

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

    // Clean up this test's file
    cleanup_log_file(&log_file);
}

#[test]
fn test_file_handler_formatting() {
    let test_dir = setup_test_dir();
    let log_file = test_dir.join("format_test.log");

    // Ensure clean state
    cleanup_log_file(&log_file);

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

    // Clean up this test's file
    cleanup_log_file(&log_file);
}
