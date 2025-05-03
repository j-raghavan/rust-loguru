use lazy_static::lazy_static;
use parking_lot::RwLock;
use std::sync::Mutex;

use rust_loguru::{
    critical, debug, error, info, log_with_metadata, success, trace, warn, LogLevel, Logger, Record,
};
use std::sync::Arc;

use rust_loguru::handler::NullHandler;
use rust_loguru::{
    critical_scope, profile_scope, resource_scope, scope,
    scope::{ResourceMetrics, ScopeGuard, ScopeType},
    scoped_info,
};
use std::thread;
use std::time::Duration;

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

#[test]
fn test_push_and_pop_context_macro() {
    use rust_loguru::{get_context, pop_context, push_context};
    push_context!("user" => "alice", "role" => "admin");
    let user = get_context!("user");
    assert_eq!(user.unwrap().to_string(), "alice");
    pop_context!();
    let user = get_context!("user");
    assert!(user.is_none());
}

#[test]
fn test_set_context_macro() {
    use rust_loguru::{get_context, pop_context, push_context, set_context};
    push_context!("foo" => "bar");
    set_context!("foo", "baz");
    let foo = get_context!("foo");
    assert_eq!(foo.unwrap().to_string(), "baz");
    pop_context!();
}

#[test]
fn test_scope_macro() {
    let result = scope!("test_scope_macro" => {
        rust_loguru::info!("Inside scope");
        42
    });
    assert_eq!(result, 42);
}

#[test]
fn test_critical_scope_macro() {
    let result = critical_scope!("critical_test" => {
        rust_loguru::info!("Inside critical scope");
        "success"
    });
    assert_eq!(result, "success");
}

#[test]
fn test_profile_scope_macro() {
    let result = profile_scope!("profile_test" => {
        thread::sleep(Duration::from_millis(10));
        let mut guard = ScopeGuard::enter("inner", ScopeType::Profiling);
        let mut metrics = ResourceMetrics::default();
        metrics.cpu_time = Duration::from_millis(100);
        metrics.memory_usage = 1024;
        metrics.io_operations = 5;
        guard.update_metrics(metrics);
        42
    });
    assert_eq!(result, 42);
}

#[test]
fn test_resource_scope_macro() {
    let result = resource_scope!("resource_test" => {
        let mut guard = ScopeGuard::enter("inner", ScopeType::Resource);
        let mut metrics = ResourceMetrics::default();
        metrics.memory_usage = 2048;
        metrics.io_operations = 10;
        guard.update_metrics(metrics);
        "resource tracked"
    });
    assert_eq!(result, "resource tracked");
}

#[test]
fn test_nested_scope_macros() {
    let result = scope!("outer" => {
        let inner_result = scope!("inner" => {
            rust_loguru::debug!("Inside inner scope");
            21
        });
        inner_result * 2
    });
    assert_eq!(result, 42);
}

#[test]
fn test_scope_logging() {
    scope!("logging_test" => {
        rust_loguru::trace!("Trace message in scope");
        rust_loguru::debug!("Debug message in scope");
        rust_loguru::info!("Info message in scope");
        rust_loguru::error!("Error message in scope");
    });
}

#[test]
#[should_panic(expected = "test panic")]
fn test_critical_scope_panic() {
    critical_scope!("critical_panic" => {
        panic!("test panic");
    });
}

#[test]
fn test_scoped_info() {
    let result = scoped_info!("scoped_info_test" => {
        rust_loguru::info!("Inside scoped info");
        "success"
    });
    assert_eq!(result, "success");
}

#[test]
fn test_log_error_macro() {
    use rust_loguru::log_error;
    let err = "something went wrong";
    log_error!(err);
    log_error!(err, "custom message");
}

#[test]
fn test_log_error_with_context_macro() {
    use rust_loguru::log_error_with_context;
    let err = "fail";
    let ctx = "context info";
    log_error_with_context!(err, ctx);
}

#[test]
fn test_try_log_macro_result() {
    use rust_loguru::try_log;
    let ok: Result<i32, &str> = Ok(1);
    let err: Result<i32, &str> = Err("fail");
    assert_eq!(try_log!(ok, "should not log"), Ok(1));
    let res = try_log!(err, "should log");
    assert!(res.is_err());
}

#[test]
fn test_try_log_macro_option() {
    use rust_loguru::try_log;
    let some = Some(42);
    let none: Option<i32> = None;
    assert_eq!(try_log!(option some, "should not log"), Some(42));
    assert_eq!(try_log!(option none, "should log"), None);
}

#[test]
fn test_log_if_enabled_macro() {
    use rust_loguru::{log_if_enabled, LogLevel, STATIC_LEVEL};
    let result = log_if_enabled!(LogLevel::Info, "Should log if enabled");
    if LogLevel::Info >= STATIC_LEVEL {
        assert!(result);
    } else {
        assert!(!result);
    }
}

#[test]
fn test_info_kv_macro() {
    let _logger = create_test_logger(LogLevel::Info);
    use rust_loguru::info_kv;
    let result = info_kv!("Structured message"; "foo" => "bar", "baz" => "qux");
    assert!(result);
}
