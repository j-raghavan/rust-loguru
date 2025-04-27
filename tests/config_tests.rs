use rust_loguru::config::{LoggerConfig, LoggerConfigBuilder};
use rust_loguru::handler::{new_handler_ref, NullHandler};
use rust_loguru::level::LogLevel;
use std::env;
// use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

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

#[test]
fn test_env_override_level() {
    env::set_var("LOGURU_LEVEL", "Debug");
    let config = LoggerConfigBuilder::new().with_env_overrides().build();
    assert_eq!(config.level, LogLevel::Debug);
    env::remove_var("LOGURU_LEVEL");
}

#[test]
fn test_env_override_capture_source() {
    env::set_var("LOGURU_CAPTURE_SOURCE", "false");
    let config = LoggerConfigBuilder::new().with_env_overrides().build();
    assert!(!config.capture_source);
    env::remove_var("LOGURU_CAPTURE_SOURCE");
}

#[test]
fn test_env_override_use_colors() {
    env::set_var("LOGURU_USE_COLORS", "false");
    let config = LoggerConfigBuilder::new().with_env_overrides().build();
    assert!(!config.use_colors);
    env::remove_var("LOGURU_USE_COLORS");
}

#[test]
fn test_env_override_format() {
    env::set_var("LOGURU_FORMAT", "[custom] {message}");
    let config = LoggerConfigBuilder::new().with_env_overrides().build();
    assert_eq!(config.format, "[custom] {message}");
    env::remove_var("LOGURU_FORMAT");
}

#[test]
fn test_env_override_combined() {
    env::set_var("LOGURU_LEVEL", "Error");
    env::set_var("LOGURU_CAPTURE_SOURCE", "true");
    env::set_var("LOGURU_USE_COLORS", "true");
    env::set_var("LOGURU_FORMAT", "{level}: {message}");
    let config = LoggerConfigBuilder::new().with_env_overrides().build();
    assert_eq!(config.level, LogLevel::Error);
    assert!(config.capture_source);
    assert!(config.use_colors);
    assert_eq!(config.format, "{level}: {message}");
    env::remove_var("LOGURU_LEVEL");
    env::remove_var("LOGURU_CAPTURE_SOURCE");
    env::remove_var("LOGURU_USE_COLORS");
    env::remove_var("LOGURU_FORMAT");
}

#[test]
fn test_toml_config_full() {
    let toml = r#"
level = "Warning"
capture_source = false
use_colors = false
format = "[TOML] {message}"
"#;
    let config = LoggerConfigBuilder::new()
        .from_toml_str(toml)
        .unwrap()
        .build();
    assert_eq!(config.level, LogLevel::Warning);
    assert!(!config.capture_source);
    assert!(!config.use_colors);
    assert_eq!(config.format, "[TOML] {message}");
}

#[test]
fn test_toml_config_partial() {
    let toml = r#"
level = "Error"
"#;
    let config = LoggerConfigBuilder::new()
        .from_toml_str(toml)
        .unwrap()
        .build();
    assert_eq!(config.level, LogLevel::Error);
    assert!(config.capture_source); // default
    assert!(config.use_colors); // default
    assert_eq!(config.format, "{time} {level} {message}"); // default
}

#[test]
fn test_toml_file_config() {
    let toml = r#"
level = "Debug"
capture_source = true
use_colors = false
format = "[file] {message}"
"#;
    let mut file = NamedTempFile::new().unwrap();
    write!(file, "{}", toml).unwrap();
    let config = LoggerConfigBuilder::new()
        .from_toml_file(file.path())
        .unwrap()
        .build();
    assert_eq!(config.level, LogLevel::Debug);
    assert!(config.capture_source);
    assert!(!config.use_colors);
    assert_eq!(config.format, "[file] {message}");
}

#[test]
fn test_toml_and_builder_precedence() {
    let toml = r#"
level = "Info"
use_colors = false
"#;
    let config = LoggerConfigBuilder::new()
        .from_toml_str(toml)
        .unwrap()
        .level(LogLevel::Error) // should override TOML
        .use_colors(true) // should override TOML
        .build();
    assert_eq!(config.level, LogLevel::Error);
    assert!(config.use_colors);
}

#[test]
fn test_toml_and_env_precedence() {
    env::set_var("LOGURU_LEVEL", "Debug");
    let toml = r#"
level = "Info"
"#;
    let config = LoggerConfigBuilder::new()
        .from_toml_str(toml)
        .unwrap()
        .with_env_overrides()
        .build();
    assert_eq!(config.level, LogLevel::Debug); // env wins
    env::remove_var("LOGURU_LEVEL");
}
