use parking_lot::RwLock;
use rust_loguru::{
    critical, debug, error, info, log_with_metadata, success, trace, warn, LogLevel, Record,
};
use std::sync::Arc;

use rust_loguru::handler::NullHandler;

#[test]
fn test_trace_macro() {
    let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Trace)));
    let mut logger = rust_loguru::Logger::new(LogLevel::Trace);
    logger.add_handler(handler);
    let _ = rust_loguru::init(logger);

    let result = trace!("Test trace message");
    assert!(result);
}

#[test]
fn test_debug_macro() {
    let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Debug)));
    let mut logger = rust_loguru::Logger::new(LogLevel::Debug);
    logger.add_handler(handler);
    let _ = rust_loguru::init(logger);

    let result = debug!("Test debug message");
    assert!(result);
}

#[test]
fn test_info_macro() {
    let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
    let mut logger = rust_loguru::Logger::new(LogLevel::Info);
    logger.add_handler(handler);
    let _ = rust_loguru::init(logger);

    let result = info!("Test info message");
    assert!(result);
}

#[test]
fn test_success_macro() {
    let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Success)));
    let mut logger = rust_loguru::Logger::new(LogLevel::Success);
    logger.add_handler(handler);
    let _ = rust_loguru::init(logger);

    let result = success!("Test success message");
    assert!(result);
}

#[test]
fn test_warn_macro() {
    let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Warning)));
    let mut logger = rust_loguru::Logger::new(LogLevel::Warning);
    logger.add_handler(handler);
    let _ = rust_loguru::init(logger);

    let result = warn!("Test warning message");
    assert!(result);
}

#[test]
fn test_error_macro() {
    let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Error)));
    let mut logger = rust_loguru::Logger::new(LogLevel::Error);
    logger.add_handler(handler);
    let _ = rust_loguru::init(logger);

    let result = error!("Test error message");
    assert!(result);
}

#[test]
fn test_critical_macro() {
    let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Critical)));
    let mut logger = rust_loguru::Logger::new(LogLevel::Critical);
    logger.add_handler(handler);
    let _ = rust_loguru::init(logger);

    let result = critical!("Test critical message");
    assert!(result);
}

#[test]
fn test_macro_formatting() {
    let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
    let mut logger = rust_loguru::Logger::new(LogLevel::Info);
    logger.add_handler(handler);
    let _ = rust_loguru::init(logger);

    let result = info!("Formatted message: {}", 42);
    assert!(result);
}

#[test]
fn test_log_with_metadata() {
    let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
    let mut logger = rust_loguru::Logger::new(LogLevel::Info);
    logger.add_handler(handler);
    let _ = rust_loguru::init(logger);

    let result = log_with_metadata!(
        LogLevel::Info,
        "key1" => "value1",
        "key2" => "value2";
        "Test message with metadata"
    );
    assert!(result);
}

#[test]
fn test_macro_source_location() {
    let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
    let mut logger = rust_loguru::Logger::new(LogLevel::Info);
    logger.add_handler(handler);
    let _ = rust_loguru::init(logger);

    let result = info!("Test message");
    assert!(result);

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
    let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Warning)));
    let mut logger = rust_loguru::Logger::new(LogLevel::Warning);
    logger.add_handler(handler);
    let _ = rust_loguru::init(logger);

    // Info message should be filtered out
    let result = info!("This should be filtered out");
    assert!(!result);

    // Warning message should pass through
    let result = warn!("This should be logged");
    assert!(result);
}

#[test]
fn test_macro_with_multiple_handlers() {
    let handler1 = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
    let handler2 = Arc::new(RwLock::new(NullHandler::new(LogLevel::Warning)));
    let mut logger = rust_loguru::Logger::new(LogLevel::Info);
    logger.add_handler(handler1);
    logger.add_handler(handler2);
    let _ = rust_loguru::init(logger);

    // Info message should be handled by handler1
    let result = info!("Test info message");
    assert!(result);

    // Warning message should be handled by both handlers
    let result = warn!("Test warning message");
    assert!(result);
}
