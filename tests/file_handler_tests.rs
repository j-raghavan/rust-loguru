use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::Once;

use rust_loguru::formatters::Formatter;
use rust_loguru::handler::file::FileHandler;
use rust_loguru::handler::Handler;
use rust_loguru::level::LogLevel;
use rust_loguru::record::Record;

// Static initialization
static INIT: Once = Once::new();
static TEST_DIR_SETUP: Mutex<bool> = Mutex::new(false);

fn get_test_dir() -> PathBuf {
    PathBuf::from("test_logs")
}

fn setup_test_environment() {
    INIT.call_once(|| {
        let test_dir = get_test_dir();

        // Clean up any existing directory
        if test_dir.exists() {
            let _ = fs::remove_dir_all(&test_dir);
        }

        // Create the directory fresh
        let _ = fs::create_dir_all(&test_dir);

        // Mark as initialized
        let mut initialized = TEST_DIR_SETUP.lock().unwrap();
        *initialized = true;
    });

    // Ensure directory exists even if another thread already ran the initialization
    let test_dir = get_test_dir();
    if !test_dir.exists() {
        let _ = fs::create_dir_all(&test_dir);
    }
}

fn get_unique_log_path(test_name: &str) -> PathBuf {
    setup_test_environment();
    get_test_dir().join(format!("{}.log", test_name))
}

#[test]
fn test_file_handler_basic() {
    let log_file = get_unique_log_path("basic_test");

    // Remove the file if it exists
    if log_file.exists() {
        let _ = fs::remove_file(&log_file);
    }

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

    let contents = match fs::read_to_string(&log_file) {
        Ok(content) => content,
        Err(e) => panic!("Failed to read log file: {}", e),
    };

    assert!(contents.contains("test message"));
}

#[test]
fn test_file_handler_level_filtering() {
    let log_file = get_unique_log_path("level_filtering_test");

    // Remove the file if it exists
    if log_file.exists() {
        let _ = fs::remove_file(&log_file);
    }

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

    let contents = match fs::read_to_string(&log_file) {
        Ok(content) => content,
        Err(e) => panic!("Failed to read log file: {}", e),
    };

    assert!(!contents.contains("info message"));
    assert!(contents.contains("warning message"));
}

#[test]
fn test_file_handler_enable_disable() {
    let log_file = get_unique_log_path("enable_disable_test");

    // Remove the file if it exists
    if log_file.exists() {
        let _ = fs::remove_file(&log_file);
    }

    let mut handler = FileHandler::new(&log_file).expect("Failed to create file handler");

    // Write while enabled
    let record1 = Record::new(
        LogLevel::Info,
        "enabled message",
        Some("test".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );
    assert!(handler.handle(&record1).is_ok());

    // Disable and write again
    handler.set_enabled(false);
    let record2 = Record::new(
        LogLevel::Info,
        "disabled message",
        Some("test".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );
    assert!(handler.handle(&record2).is_ok());

    let contents = match fs::read_to_string(&log_file) {
        Ok(content) => content,
        Err(e) => panic!("Failed to read log file: {}", e),
    };

    assert!(contents.contains("enabled message"));
    assert!(!contents.contains("disabled message"));
}

#[test]
fn test_file_handler_with_metadata() {
    let log_file = get_unique_log_path("metadata_test");

    // Remove the file if it exists
    if log_file.exists() {
        let _ = fs::remove_file(&log_file);
    }

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

    let contents = match fs::read_to_string(&log_file) {
        Ok(content) => content,
        Err(e) => panic!("Failed to read log file: {}", e),
    };

    assert!(contents.contains("key1=value1"));
    assert!(contents.contains("key2=value2"));
}

#[test]
fn test_file_handler_formatting() {
    let log_file = get_unique_log_path("formatting_test");

    // Remove the file if it exists
    if log_file.exists() {
        let _ = fs::remove_file(&log_file);
    }

    let handler = FileHandler::new(&log_file)
        .expect("Failed to create file handler")
        .with_pattern("{level} - {message}")
        .with_colors(false);

    let record = Record::new(
        LogLevel::Info,
        "Test message",
        Some("test".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );

    assert!(handler.handle(&record).is_ok());

    let contents = match fs::read_to_string(&log_file) {
        Ok(content) => content,
        Err(e) => panic!("Failed to read log file: {}", e),
    };

    assert!(contents.contains("INFO - Test message"));
}
