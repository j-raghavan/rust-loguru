use lazy_static::lazy_static;
use parking_lot::RwLock;
use std::sync::Mutex;

use rust_loguru::{
    critical, debug, error, info, log_with_metadata, success, trace, warn, LogLevel, Logger, Record,
};
use std::sync::Arc;

use rust_loguru::handler::NullHandler;

lazy_static! {
    // This mutex ensures that only one test can initialize the logger at a time
    static ref TEST_MUTEX: Mutex<()> = Mutex::new(());
}

// Add this function to create a fresh logger for each test
fn create_test_logger(level: LogLevel) -> Logger {
    // Lock the mutex during logger initialization
    let _guard = TEST_MUTEX.lock().unwrap();

    let mut logger = Logger::new(level);
    // Create a handler with the same level as the logger
    let handler = Arc::new(RwLock::new(NullHandler::new(level)));
    logger.add_handler(handler);
    // Initialize the global logger
    rust_loguru::init(logger.clone());

    // Add this line to verify the global level was set correctly
    assert_eq!(
        rust_loguru::global().read().level(),
        level,
        "Global logger level was not set correctly"
    );

    logger
}

#[test]
fn test_trace_macro() {
    let _logger = create_test_logger(LogLevel::Trace);
    // let _ = rust_loguru::init(logger);

    let result = trace!("Test trace message");
    println!("Trace macro result: {}", result);
    assert!(result, "Trace macro should return true");
}

#[test]
fn test_debug_macro() {
    let _logger = create_test_logger(LogLevel::Debug);
    // let _ = rust_loguru::init(logger);

    // Add this line to verify the global logger level
    println!(
        "Global logger level: {:?}",
        rust_loguru::global().read().level()
    );

    let result = debug!("Test debug message");
    println!("Debug macro result: {}", result);
    assert!(result, "Debug macro should return true");
}

#[test]
fn test_info_macro() {
    let _logger = create_test_logger(LogLevel::Info);
    // let _ = rust_loguru::init(logger);

    // Add debug prints to help diagnose the issue in CI
    let result = info!("Test info message");
    println!("Info macro result: {}", result);

    assert!(result, "Info macro should return true");
}

#[test]
fn test_success_macro() {
    let _logger = create_test_logger(LogLevel::Success);
    // let _ = rust_loguru::init(logger);

    let result = success!("Test success message");
    assert!(result, "Success macro should return true");
}

#[test]
fn test_warn_macro() {
    let _logger = create_test_logger(LogLevel::Warning);
    // let _ = rust_loguru::init(logger);

    let result = warn!("Test warning message");
    assert!(result, "Warning macro should return true");
}

#[test]
fn test_error_macro() {
    let _logger = create_test_logger(LogLevel::Error);
    // let _ = rust_loguru::init(logger);

    let result = error!("Test error message");
    assert!(result, "Error macro should return true");
}

#[test]
fn test_critical_macro() {
    let _logger = create_test_logger(LogLevel::Critical);
    // let _ = rust_loguru::init(logger);

    let result = critical!("Test critical message");
    assert!(result, "Critical macro should return true");
}

#[test]
fn test_macro_formatting() {
    let _logger = create_test_logger(LogLevel::Info);

    // let _ = rust_loguru::init(logger);

    // The issue might be with the format parameter - let's make sure the handler can process it
    let result = info!("Formatted message: {}", 42);
    // Debug output to help diagnose
    println!("Formatting macro result: {}", result);
    assert!(result, "Formatted macro should return true");
}

#[test]
fn test_log_with_metadata() {
    let _logger = create_test_logger(LogLevel::Info);
    // let _ = rust_loguru::init(logger);

    let result = log_with_metadata!(
        LogLevel::Info,
        "key1" => "value1",
        "key2" => "value2";
        "Test message with metadata"
    );
    println!("Log with metadata result: {}", result);
    assert!(result, "Metadata logging should return true");
}

#[test]
fn test_macro_source_location() {
    let _logger = create_test_logger(LogLevel::Info);
    // let _ = rust_loguru::init(logger);

    let result = info!("Test message");
    assert!(result, "Info macro should return true");

    // Verify that the record contains the correct source location
    let record = Record::new(
        LogLevel::Info,
        "Test message",
        Some(module_path!().to_string()),
        Some(file!().to_string()),
        Some(line!()),
    );
    assert_eq!(record.module(), module_path!());
    assert_eq!(record.file(), file!());
}

#[test]
fn test_macro_level_filtering() {
    // this both sets the global level AND adds a handler
    let _logger = create_test_logger(LogLevel::Warning);

    // Info should now be filtered out
    let result = info!("This should be filtered out");
    assert!(!result, "Info message should be filtered out");

    // Warning should pass through
    let result = warn!("This should be logged");
    assert!(result, "Warning message should pass through");
}

#[test]
fn test_macro_with_multiple_handlers() {
    let mut logger = Logger::new(LogLevel::Info);

    // Create and add the first handler with INFO level
    let handler1 = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
    logger.add_handler(handler1);

    // Create and add the second handler with WARNING level
    let handler2 = Arc::new(RwLock::new(NullHandler::new(LogLevel::Warning)));
    logger.add_handler(handler2);

    let _ = rust_loguru::init(logger.clone());

    // Info message should be handled by handler1 only
    // The return value should be true if ANY handler processed the message
    let result = info!("Test info message");
    assert!(
        result,
        "Info message should be handled by at least one handler"
    );

    // Warning message should be handled by both handlers
    let result = warn!("Test warning message");
    assert!(result, "Warning message should be handled by both handlers");
}
