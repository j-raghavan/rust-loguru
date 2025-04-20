use rust_loguru::config::{LoggerConfig, LoggerConfigBuilder};
use rust_loguru::handler::{new_handler_ref, NullHandler};
use rust_loguru::level::LogLevel;
use std::path::PathBuf;

#[test]
fn test_default_config() {
    let config = LoggerConfig::default();
    assert_eq!(config.level, LogLevel::Info);
    assert!(config.handlers.is_empty());
    assert!(config.capture_source);
    assert!(config.use_colors);
    assert_eq!(config.format, "{time} {level} {message}");
}

#[test]
fn test_builder_pattern() {
    let config = LoggerConfigBuilder::new()
        .level(LogLevel::Debug)
        .capture_source(false)
        .use_colors(false)
        .format("custom format".to_string())
        .build();

    assert_eq!(config.level, LogLevel::Debug);
    assert!(!config.capture_source);
    assert!(!config.use_colors);
    assert_eq!(config.format, "custom format");
}

#[test]
fn test_basic_console_config() {
    let config = LoggerConfig::basic_console();
    assert_eq!(config.level, LogLevel::Info);
    assert!(config.use_colors);
    assert!(config.capture_source);
}

#[test]
fn test_file_logging_config() {
    let path = PathBuf::from("test.log");
    let config = LoggerConfig::file_logging(path);
    assert_eq!(config.level, LogLevel::Info);
    assert!(!config.use_colors);
}

#[test]
fn test_development_config() {
    let config = LoggerConfig::development();
    assert_eq!(config.level, LogLevel::Debug);
    assert!(config.use_colors);
    assert!(config.capture_source);
    assert_eq!(config.format, "{time} {level} {file}:{line} {message}");
}

#[test]
fn test_production_config() {
    let config = LoggerConfig::production();
    assert_eq!(config.level, LogLevel::Info);
    assert!(!config.use_colors);
    assert!(!config.capture_source);
    assert_eq!(config.format, "{time} {level} {message}");
}

#[test]
fn test_config_clone() {
    let config1 = LoggerConfig::development();
    let config2 = config1.clone();
    assert_eq!(config1.level, config2.level);
    assert_eq!(config1.use_colors, config2.use_colors);
    assert_eq!(config1.capture_source, config2.capture_source);
    assert_eq!(config1.format, config2.format);
}

#[test]
fn test_builder_chain() {
    let handler = new_handler_ref(NullHandler::new(LogLevel::Warning));
    let config = LoggerConfigBuilder::new()
        .level(LogLevel::Warning)
        .capture_source(true)
        .use_colors(true)
        .format("chain test".to_string())
        .add_handler(handler)
        .build();

    assert_eq!(config.level, LogLevel::Warning);
    assert!(config.capture_source);
    assert!(config.use_colors);
    assert_eq!(config.format, "chain test");
    assert_eq!(config.handlers.len(), 1);
}
