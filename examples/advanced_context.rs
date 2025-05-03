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

    // Set global context for the application
    context::set_global_context_value(
        "app_name",
        context::ContextValue::String("example-app".to_string()),
    );

    // Use context scope for automatic cleanup
    {
        let _scope = context::create_context_scope();
        context::set_context_value(
            "user_id",
            context::ContextValue::String("alice".to_string()),
        );

        // Log a message; context will be attached if integrated into Record/formatters
        info!("User logged in");

        // Nested context scope for request handling
        {
            let _req_scope = context::create_context_scope();
            context::set_context_value(
                "request_id",
                context::ContextValue::String("req-123".to_string()),
            );

            debug!("Processing request");
        } // request context is automatically cleared here
    } // user context is automatically cleared here

    // --- Async context propagation example ---
    {
        let _scope = context::create_context_scope();
        context::set_context_value(
            "trace_id",
            context::ContextValue::String("abc123".to_string()),
        );
        let snapshot = context::create_context_snapshot();

        let handle = thread::spawn(move || {
            let _scope = context::create_context_scope();
            context::restore_context(&snapshot);
            info!("Logging from another thread with propagated context");
        });
        handle.join().unwrap();
    }

    // Demonstrate complex context values
    {
        let _scope = context::create_context_scope();
        let mut user_data = context::ContextMap::new();
        user_data.insert(
            "name".to_string(),
            context::ContextValue::String("alice".to_string()),
        );
        user_data.insert("age".to_string(), context::ContextValue::Integer(30));
        user_data.insert(
            "roles".to_string(),
            context::ContextValue::Array(vec![
                context::ContextValue::String("admin".to_string()),
                context::ContextValue::String("user".to_string()),
            ]),
        );

        context::set_context_value("user", context::ContextValue::Map(user_data));
        info!("User data attached to context");
    }
}
