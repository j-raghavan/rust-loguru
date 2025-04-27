use parking_lot::RwLock;
use rust_loguru::handler::console::ConsoleHandler;
use rust_loguru::{context, debug, info, LogLevel, Logger};
use std::sync::Arc;
use std::thread;

fn main() {
    // Set up a simple logger for demonstration
    let handler = Arc::new(RwLock::new(
        ConsoleHandler::stderr(LogLevel::Debug).with_colors(true),
    ));
    let mut logger = Logger::new(LogLevel::Debug);
    logger.add_handler(handler);
    rust_loguru::init(logger);

    // Add context for the current thread (e.g., user ID)
    let mut ctx = context::ContextMap::new();
    ctx.insert(
        "user_id".to_string(),
        context::ContextValue::String("alice".to_string()),
    );
    context::push_context(ctx);

    // Log a message; context will be attached if integrated into Record/formatters
    info!("User logged in");

    // Add more context (e.g., request ID) in a nested scope
    let mut req_ctx = context::ContextMap::new();
    req_ctx.insert(
        "request_id".to_string(),
        context::ContextValue::String("req-123".to_string()),
    );
    context::push_context(req_ctx);

    debug!("Processing request");

    // Pop request context when done
    context::pop_context();

    // Pop user context at the end
    context::pop_context();

    // --- Async context propagation example ---
    let mut ctx = context::ContextMap::new();
    ctx.insert(
        "trace_id".to_string(),
        context::ContextValue::String("abc123".to_string()),
    );
    context::push_context(ctx);
    let arc_ctx = context::propagate_context_for_async();

    let handle = thread::spawn(move || {
        context::set_context_from_arc(arc_ctx);
        info!("Logging from another thread with propagated context");
        context::pop_context();
    });
    handle.join().unwrap();
    context::pop_context();
}
