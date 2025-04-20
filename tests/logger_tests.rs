use parking_lot::RwLock;
use rust_loguru::handler::NullHandler;
use rust_loguru::{Handler, LogLevel, Logger, Record};
use std::sync::Arc;

#[test]
fn test_logger_initialization() {
    let logger = Logger::new(LogLevel::Info);
    assert_eq!(logger.level(), LogLevel::Info);
}

#[test]
fn test_logger_handler_registration() {
    let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
    let logger = Logger::new(LogLevel::Debug).add_handler(handler.clone());

    // Test by logging a message
    let record = Record::new(LogLevel::Info, "Test message", "test_module", "test.rs", 42);
    assert!(logger.log(&record));
}

#[test]
fn test_logger_level_filtering() {
    let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
    let logger = Logger::new(LogLevel::Info).add_handler(handler.clone());

    // Test with level above minimum
    let record = Record::new(LogLevel::Info, "Test message", "test_module", "test.rs", 42);
    assert!(logger.log(&record));

    // Test with level below minimum
    let record = Record::new(
        LogLevel::Debug,
        "Test message",
        "test_module",
        "test.rs",
        42,
    );
    assert!(!logger.log(&record));
}

#[test]
fn test_logger_handler_level_filtering() {
    let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Warning)));
    let logger = Logger::new(LogLevel::Debug).add_handler(handler.clone());

    // Test with level above handler minimum
    let record = Record::new(
        LogLevel::Warning,
        "Test message",
        "test_module",
        "test.rs",
        42,
    );
    assert!(logger.log(&record));

    // Test with level below handler minimum
    let record = Record::new(LogLevel::Info, "Test message", "test_module", "test.rs", 42);
    assert!(!logger.log(&record));
}

#[test]
fn test_logger_disabled_handler() {
    let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
    {
        let mut handler_guard = handler.write();
        handler_guard.set_enabled(false);
    }
    let logger = Logger::new(LogLevel::Debug).add_handler(handler.clone());

    let record = Record::new(LogLevel::Info, "Test message", "test_module", "test.rs", 42);
    assert!(!logger.log(&record));
}

#[test]
fn test_global_logger() {
    let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
    let logger = Logger::new(LogLevel::Debug).add_handler(handler.clone());
    rust_loguru::init(logger);

    let record = Record::new(LogLevel::Info, "Test message", "test_module", "test.rs", 42);
    assert!(rust_loguru::log(&record));
}

#[test]
fn test_logger_multiple_handlers() {
    let handler1 = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
    let handler2 = Arc::new(RwLock::new(NullHandler::new(LogLevel::Warning)));
    let logger = Logger::new(LogLevel::Debug)
        .add_handler(handler1.clone())
        .add_handler(handler2.clone());

    // Test with Warning level (should be handled by both handlers)
    let record = Record::new(
        LogLevel::Warning,
        "Test message",
        "test_module",
        "test.rs",
        42,
    );
    assert!(logger.log(&record));

    // Test with Info level (should be handled by handler1 only)
    let record = Record::new(LogLevel::Info, "Test message", "test_module", "test.rs", 42);
    assert!(logger.log(&record));
}

#[test]
fn test_logger_level_modification() {
    let mut logger = Logger::new(LogLevel::Info);
    assert_eq!(logger.level(), LogLevel::Info);

    logger.set_level(LogLevel::Debug);
    assert_eq!(logger.level(), LogLevel::Debug);
}

#[test]
fn test_logger_with_metadata() {
    let logger = Logger::new(LogLevel::Info);
    let record = Record::new(
        LogLevel::Info,
        "Test message with metadata",
        "test_module",
        "test.rs",
        42,
    )
    .with_metadata("key1", "value1")
    .with_metadata("key2", "value2");

    let metadata = record.metadata();
    assert_eq!(metadata.len(), 2);
    assert_eq!(metadata.get("key1"), Some(&"value1".to_string()));
    assert_eq!(metadata.get("key2"), Some(&"value2".to_string()));
}
