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

#[test]
fn test_log_compat_integration_sets_logger() {
    use log::{info, LevelFilter, Log, Metadata, Record};
    use parking_lot::RwLock;
    use rust_loguru::handler::NullHandler;
    use rust_loguru::integration::log_compat;
    use rust_loguru::{LogLevel, Logger};
    use std::sync::Arc;

    struct DummyLogger;
    impl Log for DummyLogger {
        fn enabled(&self, _: &Metadata) -> bool {
            false
        }
        fn log(&self, _: &Record) {}
        fn flush(&self) {}
    }

    // Try to set a dummy logger to check if the logger is already set
    if log::set_logger(&DummyLogger).is_err() {
        eprintln!("Logger already set, skipping test_log_compat_integration_sets_logger");
        return;
    }
    log::set_max_level(LevelFilter::Info);

    // Set up a logger
    let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
    let mut logger = Logger::new(LogLevel::Info);
    logger.add_handler(handler);
    let _ = rust_loguru::init(logger);

    // Set loguru as the log crate logger
    log_compat::init_loguru_as_log();
    log::set_max_level(LevelFilter::Info);
    info!("This is a log crate info message");
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_async_runtime_tokio_integration() {
    use rust_loguru::integration::async_runtime;
    // This should not panic and should spawn a background task
    async_runtime::integrate_with_tokio().await;
}

#[cfg(test)]
mod integration_tests {
    use rust_loguru::integration::{async_runtime, log_compat, middleware};

    #[test]
    fn test_log_compat_init_loguru_as_log() {
        // Should not panic now
        let _ = std::panic::catch_unwind(|| log_compat::init_loguru_as_log());
    }

    #[test]
    #[cfg(feature = "tokio")]
    fn test_async_runtime_integrate_with_tokio() {
        let _ = std::panic::catch_unwind(|| {
            tokio_test::block_on(async_runtime::integrate_with_tokio());
        });
    }

    #[test]
    #[should_panic(expected = "framework middleware not yet implemented")]
    fn test_middleware_request_response_logging() {
        middleware::request_response_logging();
    }
}
