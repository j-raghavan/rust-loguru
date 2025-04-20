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
    let mut logger = Logger::new(LogLevel::Info);
    let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
    logger.add_handler(handler.clone());

    let record = Record::new(
        LogLevel::Info,
        "Test message",
        Some("test_module".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );
    assert!(logger.log(&record));
}

#[test]
fn test_logger_level_filtering() {
    let mut logger = Logger::new(LogLevel::Warning);
    let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
    logger.add_handler(handler);

    let record = Record::new(
        LogLevel::Info,
        "Test message",
        Some("test_module".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );
    assert!(!logger.log(&record));

    let record = Record::new(
        LogLevel::Warning,
        "Test message",
        Some("test_module".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );
    assert!(logger.log(&record));
}

#[test]
fn test_logger_handler_filtering() {
    let mut logger = Logger::new(LogLevel::Info);
    let mut handler = NullHandler::new(LogLevel::Info);
    handler.set_level(LogLevel::Warning);
    let handler = Arc::new(RwLock::new(handler));
    logger.add_handler(handler);

    let record = Record::new(
        LogLevel::Info,
        "Test message",
        Some("test_module".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );
    assert!(!logger.log(&record));

    let record = Record::new(
        LogLevel::Warning,
        "Test message",
        Some("test_module".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );
    assert!(logger.log(&record));
}

#[test]
fn test_logger_disabled_handler() {
    let mut logger = Logger::new(LogLevel::Info);
    let mut handler = NullHandler::new(LogLevel::Info);
    handler.set_enabled(false);
    let handler = Arc::new(RwLock::new(handler));
    logger.add_handler(handler);

    let record = Record::new(
        LogLevel::Info,
        "Test message",
        Some("test_module".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );
    assert!(!logger.log(&record));
}

#[test]
fn test_global_logger() {
    let mut logger = Logger::new(LogLevel::Info);
    let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
    logger.add_handler(handler);
    let logger = rust_loguru::init(logger);

    let record = Record::new(
        LogLevel::Info,
        "Test message",
        Some("test_module".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );
    assert!(logger.log(&record));
}

#[test]
fn test_logger_multiple_handlers() {
    let mut logger = Logger::new(LogLevel::Info);
    let handler1 = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
    let handler2 = Arc::new(RwLock::new(NullHandler::new(LogLevel::Warning)));
    logger.add_handler(handler1);
    logger.add_handler(handler2);

    let record = Record::new(
        LogLevel::Info,
        "Test message",
        Some("test_module".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );
    assert!(logger.log(&record));

    let record = Record::new(
        LogLevel::Warning,
        "Test message",
        Some("test_module".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );
    assert!(logger.log(&record));
}

#[test]
fn test_logger_level_modification() {
    let mut logger = Logger::new(LogLevel::Info);
    assert_eq!(logger.level(), LogLevel::Info);

    logger.set_level(LogLevel::Warning);
    assert_eq!(logger.level(), LogLevel::Warning);
}

#[test]
fn test_logger_with_metadata() {
    let mut logger = Logger::new(LogLevel::Info);
    let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
    logger.add_handler(handler);

    let record = Record::new(
        LogLevel::Info,
        "Test message",
        Some("test_module".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    )
    .with_metadata("key", "value");
    assert!(logger.log(&record));
}
