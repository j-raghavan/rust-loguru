use std::fs;
use std::io;
use tempfile::tempdir;

use rust_loguru::handler::file::FileHandler;
use rust_loguru::handler::Handler;
use rust_loguru::level::LogLevel;
use rust_loguru::record::Record;

#[test]
fn test_file_handler_creation() -> io::Result<()> {
    let temp_dir = tempdir()?;
    let log_path = temp_dir.path().join("test.log");
    let handler = FileHandler::new(log_path.to_str().unwrap())?;

    assert_eq!(handler.level(), LogLevel::Info);
    assert!(handler.enabled());
    assert_eq!(handler.path(), log_path.to_str().unwrap());
    Ok(())
}

#[test]
fn test_file_handler_level_filtering() -> io::Result<()> {
    let temp_dir = tempdir()?;
    let log_path = temp_dir.path().join("test.log");
    let mut handler = FileHandler::new(log_path.to_str().unwrap())?;
    handler.set_level(LogLevel::Warning);

    let info_record = Record::new(
        LogLevel::Info,
        "Info message",
        Some("test_module".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );
    let warning_record = Record::new(
        LogLevel::Warning,
        "Warning message",
        Some("test_module".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );

    assert!(!handler.handle(&info_record));
    assert!(handler.handle(&warning_record));

    let contents = fs::read_to_string(log_path)?;
    assert!(!contents.contains("Info message"));
    assert!(contents.contains("Warning message"));
    Ok(())
}

#[test]
fn test_file_handler_enable_disable() -> io::Result<()> {
    let temp_dir = tempdir()?;
    let log_path = temp_dir.path().join("test.log");
    let mut handler = FileHandler::new(log_path.to_str().unwrap())?;
    assert!(handler.enabled());

    handler.set_enabled(false);
    assert!(!handler.enabled());

    let record = Record::new(
        LogLevel::Info,
        "Test message",
        Some("test_module".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );

    assert!(!handler.handle(&record));
    let contents = fs::read_to_string(log_path)?;
    assert!(contents.is_empty());
    Ok(())
}

#[test]
fn test_file_handler_write() -> io::Result<()> {
    let temp_dir = tempdir()?;
    let log_path = temp_dir.path().join("test.log");
    let mut handler = FileHandler::new(log_path.to_str().unwrap())?;

    let record = Record::new(
        LogLevel::Info,
        "test message",
        Some("test".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );

    assert!(handler.handle(&record));

    let contents = fs::read_to_string(&log_path)?;
    assert!(contents.contains("test message"));
    assert!(contents.contains("test.rs:42"));
    Ok(())
}

#[test]
fn test_file_handler_rotation() -> io::Result<()> {
    let temp_dir = tempdir()?;
    let log_path = temp_dir.path().join("test.log");
    let mut handler = FileHandler::new(log_path.to_str().unwrap())?
        .with_rotation(1000) // 1KB rotation size
        .with_retention(2); // Keep 2 old files

    // Write enough data to trigger rotation
    for i in 0..100 {
        let record = Record::new(
            LogLevel::Info,
            format!("Test message {}", i),
            Some("test".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );
        assert!(handler.handle(&record));
    }

    // Check that rotation occurred
    let log_files: Vec<_> = fs::read_dir(temp_dir.path())?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "log" {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    assert!(log_files.len() <= 3); // Current file + 2 old files
    Ok(())
}

#[test]
fn test_file_handler_retention() -> io::Result<()> {
    let temp_dir = tempdir()?;
    let log_path = temp_dir.path().join("test.log");
    let mut handler = FileHandler::new(log_path.to_str().unwrap())?
        .with_rotation(1000) // 1KB rotation size
        .with_retention(2); // Keep only 2 old files

    // Write enough data to trigger multiple rotations
    for i in 0..300 {
        let record = Record::new(
            LogLevel::Info,
            &format!("test message {}", i),
            Some("test".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );
        handler.handle(&record);
    }

    // Check that only the specified number of old files are kept
    let files: Vec<_> = fs::read_dir(temp_dir.path())?
        .filter_map(|entry| entry.ok())
        .collect();
    assert!(files.len() <= 3); // Current file + 2 old files
    Ok(())
}

#[test]
fn test_file_handler_with_metadata() -> io::Result<()> {
    let temp_dir = tempdir()?;
    let log_path = temp_dir.path().join("test.log");
    let mut handler = FileHandler::new(log_path.to_str().unwrap())?;

    let record = Record::new(
        LogLevel::Info,
        "Test message",
        Some("test".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );

    assert!(handler.handle(&record));
    let contents = fs::read_to_string(log_path)?;
    assert!(contents.contains("INFO"));
    assert!(contents.contains("Test message"));
    assert!(contents.contains("test.rs:42"));
    Ok(())
}

#[test]
fn test_file_handler_formatting() -> io::Result<()> {
    let temp_dir = tempdir()?;
    let log_path = temp_dir.path().join("test.log");
    let mut handler =
        FileHandler::new(log_path.to_str().unwrap())?.with_pattern("{level} - {message}");

    let record = Record::new(
        LogLevel::Info,
        "Test message",
        Some("test_module".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );

    assert!(handler.handle(&record));
    let contents = fs::read_to_string(log_path)?;
    assert!(contents.contains("INFO - Test message"));
    Ok(())
}

#[test]
fn test_file_handler_metadata() -> io::Result<()> {
    let temp_dir = tempdir()?;
    let log_path = temp_dir.path().join("test.log");
    let mut handler = FileHandler::new(log_path.to_str().unwrap())?;

    let record = Record::new(
        LogLevel::Info,
        "Metadata test",
        Some("test_module".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    )
    .with_metadata("key1", "value1")
    .with_metadata("key2", "value2");

    assert!(handler.handle(&record));
    let contents = fs::read_to_string(log_path)?;
    println!("Actual output: {}", contents);
    assert!(contents.contains("[key1=value1, key2=value2]"));
    Ok(())
}

#[test]
fn test_file_handler_structured_data() -> io::Result<()> {
    let temp_dir = tempdir()?;
    let log_path = temp_dir.path().join("test.log");
    let mut handler = FileHandler::new(log_path.to_str().unwrap())?;

    let data = serde_json::json!({
        "user_id": 123,
        "action": "login"
    });

    let record = Record::new(
        LogLevel::Info,
        "Structured data test",
        Some("test_module".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    )
    .with_structured_data("data", &data)
    .unwrap();

    assert!(handler.handle(&record));
    let contents = fs::read_to_string(log_path)?;
    assert!(contents.contains("data="));
    assert!(contents.contains(r#""user_id":123"#));
    assert!(contents.contains(r#""action":"login""#));
    Ok(())
}

#[test]
fn test_file_handler_write_error() -> io::Result<()> {
    let temp_dir = tempdir()?;
    let log_path = temp_dir.path().join("test.log");
    let mut handler = FileHandler::new(log_path.to_str().unwrap())?;

    // Close the file handle before making it read-only
    handler.close();

    // Make the file read-only
    let mut perms = fs::metadata(&log_path)?.permissions();
    perms.set_readonly(true);
    fs::set_permissions(&log_path, perms)?;

    let record = Record::new(
        LogLevel::Info,
        "Test message",
        Some("test_module".to_string()),
        Some("test.rs".to_string()),
        Some(42),
    );

    assert!(!handler.handle(&record));
    Ok(())
}
