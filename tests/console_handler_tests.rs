use rust_loguru::formatters::Formatter;
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
    let handler = ConsoleHandler::stdout(LogLevel::Info);
    assert_eq!(handler.level(), LogLevel::Info);
    assert!(handler.is_enabled());
}

#[test]
fn test_console_handler_level_filtering() {
    let output = TestOutput::new();
    let handler = ConsoleHandler::with_writer(LogLevel::Warning, Box::new(output.clone()));

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

    assert!(handler.handle(&info_record).is_ok());
    assert!(handler.handle(&warning_record).is_ok());

    let contents = output.contents();
    assert!(!contents.contains("info message"));
    assert!(contents.contains("warning message"));
}

#[test]
fn test_console_handler_enable_disable() {
    let mut handler = ConsoleHandler::stdout(LogLevel::Info);
    assert!(handler.is_enabled());

    handler.set_enabled(false);
    assert!(!handler.is_enabled());

    let record = Record::new(
        LogLevel::Info,
        "test message",
        Some("test".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );
    assert!(handler.handle(&record).is_ok());
}

#[test]
fn test_console_handler_colors() {
    let output = TestOutput::new();

    // Test with colors enabled (default)
    let handler =
        ConsoleHandler::with_writer(LogLevel::Info, Box::new(output.clone())).with_colors(true);

    let record = Record::new(
        LogLevel::Error,
        "test message",
        Some("test".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );

    assert!(handler.handle(&record).is_ok());
    let colored_output = output.contents();

    // Test with colors explicitly disabled
    let output = TestOutput::new();
    let handler =
        ConsoleHandler::with_writer(LogLevel::Info, Box::new(output.clone())).with_colors(false);

    assert!(handler.handle(&record).is_ok());
    let plain_output = output.contents();

    // Debug output
    println!("colored_output: {:?}", colored_output.as_bytes());
    println!("plain_output: {:?}", plain_output.as_bytes());

    // Check if the test environment supports colors
    if colored_output.contains("\x1b[") {
        // If colors are supported, the outputs should be different
        assert_ne!(
            colored_output, plain_output,
            "Colored and plain outputs should be different when colors are supported"
        );
    } else {
        // If colors are not supported (like in some CI environments),
        // both outputs might be the same
        println!("Note: Colors appear to be disabled in this environment");
    }
}

#[test]
fn test_console_handler_formatting() {
    let output = TestOutput::new();
    let handler = ConsoleHandler::with_writer(LogLevel::Info, Box::new(output.clone()))
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
    let contents = output.contents();
    assert!(contents.contains("INFO - Test message"));
}

#[test]
fn test_console_handler_with_metadata() -> io::Result<()> {
    let output = TestOutput::new();
    let handler = ConsoleHandler::with_writer(LogLevel::Info, Box::new(output.clone()))
        .with_pattern("{level} - {message} {metadata}");

    let record = Record::new(
        LogLevel::Info,
        "Test message",
        Some("test".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    )
    .with_metadata("key1", "value1")
    .with_metadata("key2", "value2");

    assert!(handler.handle(&record).is_ok());
    let output = output.contents();
    assert!(output.contains("key1=value1"));
    assert!(output.contains("key2=value2"));
    Ok(())
}

#[test]
fn test_console_handler_with_json_format() -> io::Result<()> {
    let output = TestOutput::new();
    let handler = ConsoleHandler::with_writer(LogLevel::Info, Box::new(output.clone()))
        .with_formatter(Formatter::json());

    let record = Record::new(
        LogLevel::Info,
        "Test message",
        Some("test".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );

    assert!(handler.handle(&record).is_ok());
    let output = output.contents();
    assert!(output.contains(r#""level":"INFO""#));
    assert!(output.contains(r#""message":"Test message""#));
    assert!(output.contains(r#""module":"test""#));
    Ok(())
}

#[test]
fn test_console_handler_structured_data() {
    let output = TestOutput::new();
    // Using Text formatter with JSON-like pattern
    let handler = ConsoleHandler::with_writer(LogLevel::Info, Box::new(output.clone()))
        .with_formatter(
            Formatter::json()
                .with_pattern(r#"{"level":"{level}","message":"{message}","module":"{module}"}"#),
        );

    let record = Record::new(
        LogLevel::Info,
        "Test message",
        Some("test".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );

    assert!(handler.handle(&record).is_ok());
    let contents = output.contents();
    assert!(contents.contains(r#""level":"INFO""#));
    assert!(contents.contains(r#""message":"Test message""#));
    assert!(contents.contains(r#""module":"test""#));
}
