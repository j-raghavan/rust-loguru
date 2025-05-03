//! Example: Advanced Macro Usage in rust-loguru
// Demonstrates context, scope, error, compile-time filtering, and structured data macros.

use parking_lot::RwLock;
use rust_loguru::handler::console::ConsoleHandler;
use rust_loguru::{
    critical, critical_scope, debug, error, get_context, info, info_kv, init, log_error,
    log_error_with_context, log_if_enabled, pop_context, profile_scope, push_context,
    resource_scope, scope,
    scope::{ScopeGuard, ScopeType},
    scoped_info, set_context, success, trace, try_log, warn, LogLevel, Logger,
};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    // Setup logger
    let handler = Arc::new(RwLock::new(
        ConsoleHandler::stderr(LogLevel::Trace).with_colors(true),
    ));
    let mut logger = Logger::new(LogLevel::Trace);
    logger.add_handler(handler);
    init(logger);

    // --- Context Macros ---
    push_context!("user" => "alice", "role" => "admin");
    set_context!("user", "bob");
    let user = get_context!("user");
    info!("Current user from context: {}", user.unwrap());
    pop_context!();

    // --- Scope Macros ---
    let result = scope!("outer_scope" => {
        info!("Inside outer scope");

        // Nested scope
        let inner_result = scope!("inner_scope" => {
            debug!("Inside inner scope");
            thread::sleep(Duration::from_millis(100));
            42
        });

        inner_result * 2
    });

    println!("Result: {}", result);

    // Critical scope with error handling
    let _ = critical_scope!("critical_operation" => {
        info!("Starting critical operation");
        if false { // Simulated error condition
            error!("Critical operation failed");
            return Err("Operation failed");
        }
        Ok(())
    });

    // Profiling scope
    let _ = profile_scope!("performance_critical" => {
        let mut guard = ScopeGuard::enter("compute", ScopeType::Profiling);

        // Simulate some work
        thread::sleep(Duration::from_millis(50));

        // Update metrics
        let mut metrics = rust_loguru::scope::ResourceMetrics::default();
        metrics.cpu_time = Duration::from_millis(50);
        metrics.memory_usage = 1024 * 1024; // 1MB
        metrics.io_operations = 10;
        guard.update_metrics(metrics);
    });

    // Resource tracking scope
    let _ = resource_scope!("resource_intensive" => {
        let mut guard = ScopeGuard::enter("file_processing", ScopeType::Resource);

        // Simulate file processing
        thread::sleep(Duration::from_millis(200));

        // Update resource usage
        let mut metrics = rust_loguru::scope::ResourceMetrics::default();
        metrics.memory_usage = 2 * 1024 * 1024; // 2MB
        metrics.io_operations = 100;
        guard.update_metrics(metrics);
    });

    // --- Error Integration Macros ---
    let err = "something went wrong";
    log_error!(err);
    log_error!(err, "custom error message");
    log_error_with_context!(err, "context info");

    let res: Result<i32, &str> = Err("fail");
    let _ = try_log!(res, "operation failed");
    let opt: Option<i32> = None;
    let _ = try_log!(option opt, "missing value");

    // --- Compile-time Level Filtering ---
    let _ = log_if_enabled!(
        LogLevel::Info,
        "This will only log if enabled at compile time"
    );

    // --- Structured Data in Level Macros ---
    info_kv!("User login event"; "user_id" => "123", "ip" => "192.168.1.1");

    // --- Level-specific macros ---
    trace!("Trace message");
    debug!("Debug message");
    info!("Info message");
    success!("Success message");
    warn!("Warning message");
    error!("Error message");
    critical!("Critical message");
}
