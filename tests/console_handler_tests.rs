use rust_loguru::handler::console::ConsoleHandler;
use rust_loguru::handler::Handler;
use rust_loguru::level::LogLevel;
use rust_loguru::record::Record;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

struct TestOutput {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl Clone for TestOutput {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
        }
    }
}

impl Write for TestOutput {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl TestOutput {
    fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn contents(&self) -> String {
        let buffer = self.buffer.lock().unwrap();
        String::from_utf8_lossy(&buffer).to_string()
    }
}

#[test]
fn test_console_handler_creation() {
    let handler = ConsoleHandler::new();
    assert_eq!(handler.level(), LogLevel::Info);
    assert!(handler.enabled());
}

#[test]
fn test_console_handler_level_filtering() {
    let mut handler = ConsoleHandler::new();
    handler.set_level(LogLevel::Warning);

    let info_record = Record::new(
        LogLevel::Info,
        "info message",
        Some("test".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );
    let warning_record = Record::new(
        LogLevel::Warning,
        "warning message",
        Some("test".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );

    assert!(!handler.handle(&info_record));
    assert!(handler.handle(&warning_record));
}

#[test]
fn test_console_handler_enable_disable() {
    let mut handler = ConsoleHandler::new();
    assert!(handler.enabled());

    handler.set_enabled(false);
    assert!(!handler.enabled());

    let record = Record::new(
        LogLevel::Info,
        "test message",
        Some("test".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );
    assert!(!handler.handle(&record));
}

#[test]
fn test_console_handler_colors() {
    let mut handler = ConsoleHandler::new();
    assert!(handler.formatter().use_colors);

    handler.set_formatter(handler.formatter().clone().with_colors(false));
    assert!(!handler.formatter().use_colors);
}

#[test]
fn test_console_handler_formatting() {
    let handler = ConsoleHandler::new();
    let record = Record::new(
        LogLevel::Error,
        "test message",
        Some("test".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );

    let formatted = handler.formatter().format(&record);
    assert!(formatted.contains("ERROR"));
    assert!(formatted.contains("test.rs:42"));
    assert!(formatted.contains("test message"));
}

#[test]
fn test_console_handler_with_custom_format() -> io::Result<()> {
    let output = TestOutput::new();
    let mut handler = ConsoleHandler::with_writer(Box::new(output.clone()))
        .with_format(|record| format!("CUSTOM: {}", record.message()));

    let record = Record::new(
        LogLevel::Info,
        "Test message",
        Some("test".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );

    assert!(handler.handle(&record));
    assert!(output.contents().contains("CUSTOM: Test message"));
    Ok(())
}

#[test]
fn test_console_handler_with_metadata() -> io::Result<()> {
    let output = TestOutput::new();
    let mut handler = ConsoleHandler::with_writer(Box::new(output.clone()));

    let record = Record::new(
        LogLevel::Info,
        "Test message",
        Some("test".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );

    assert!(handler.handle(&record));
    let output = output.contents();
    assert!(output.contains("INFO"));
    assert!(output.contains("Test message"));
    Ok(())
}

#[test]
fn test_console_handler_with_json_format() -> io::Result<()> {
    let output = TestOutput::new();
    let mut handler = ConsoleHandler::with_writer(Box::new(output.clone())).with_format(|record| {
        format!(
            r#"{{"level":"{}","message":"{}","module":"{}"}}"#,
            record.level(),
            record.message(),
            record.module()
        )
    });

    let record = Record::new(
        LogLevel::Info,
        "Test message",
        Some("test".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );

    assert!(handler.handle(&record));
    let output = output.contents();
    assert!(output.contains(r#""level":"INFO""#));
    assert!(output.contains(r#""message":"Test message""#));
    assert!(output.contains(r#""module":"test""#));
    Ok(())
}
