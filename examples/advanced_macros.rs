//! Example: Advanced Macro Usage in rust-loguru
// Demonstrates context, scope, error, compile-time filtering, and structured data macros.

use parking_lot::RwLock;
use rust_loguru::handler::console::ConsoleHandler;
use rust_loguru::{
    critical, debug, error, get_context, info, info_kv, init, log_error, log_error_with_context,
    log_if_enabled, pop_context, push_context, scope, scoped_info, set_context, success, trace,
    try_log, warn, LogLevel, Logger,
};
use std::sync::Arc;

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
    {
        let _guard = scope!("outer_scope");
        debug!("Inside outer scope");
        {
            let _scope = scoped_info!("inner_scope");
            success!("Doing some work in the inner scope");
        }
    }

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
