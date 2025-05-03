use rand::random;
use rust_loguru::handler::console::ConsoleHandler;
use rust_loguru::handler::file::FileHandler;
use rust_loguru::handler::new_handler_ref;
use rust_loguru::level::LogLevel;
use rust_loguru::logger::Logger;
use rust_loguru::Record;
use rust_loguru::{
    debug, error, info,
    scope::{ScopeGuard, ScopeType},
};

use std::thread;
use std::time::Duration;

fn main() {
    // Create console handler
    let console = ConsoleHandler::stdout(LogLevel::Debug).with_colors(true);
    let console = new_handler_ref(console);

    // Create a file handler with rotation
    let file = FileHandler::new("app.log")
        .expect("Failed to create file handler")
        .with_rotation(10 * 1024 * 1024, 5); // 10MB rotation size, keep 5 files
    let file = new_handler_ref(file);

    // Create a new logger
    let mut logger = Logger::new(LogLevel::Debug);
    logger.add_handler(console);
    logger.add_handler(file);

    // Initialize the global logger
    let mut logger = rust_loguru::init(logger);

    // Log some messages
    logger.info("Application started");
    logger.warn("This is a warning");
    logger.error("This is an error");

    // Log messages at different levels
    logger.log(&Record::new(
        LogLevel::Trace,
        "This is a trace message",
        None,
        None,
        None,
    ));
    logger.log(&Record::new(
        LogLevel::Debug,
        "This is a debug message",
        None,
        None,
        None,
    ));
    logger.log(&Record::new(
        LogLevel::Info,
        "This is an info message",
        None,
        None,
        None,
    ));
    logger.log(&Record::new(
        LogLevel::Warning,
        "This is a warning message",
        None,
        None,
        None,
    ));
    logger.log(&Record::new(
        LogLevel::Error,
        "This is an error message",
        None,
        None,
        None,
    ));
    logger.log(&Record::new(
        LogLevel::Critical,
        "This is a critical message",
        None,
        None,
        None,
    ));

    // Log with structured data
    logger.log(
        &Record::new(LogLevel::Info, "User logged in", None, None, None)
            .with_metadata("user_id", "123")
            .with_metadata("ip", "192.168.1.1"),
    );

    // Log with error context
    let result: Result<(), &str> = Err("Failed to connect to database");
    if let Err(e) = result {
        logger.log(
            &Record::new(
                LogLevel::Error,
                "Database operation failed",
                None,
                None,
                None,
            )
            .with_metadata("error", e),
        );
    }

    // Demonstrate concurrent logging
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let logger = logger.clone();
            thread::spawn(move || {
                for j in 0..10 {
                    logger.log(
                        &Record::new(
                            LogLevel::Info,
                            format!("Thread {}: Message {}", i, j),
                            None,
                            None,
                            None,
                        )
                        .with_metadata("thread_id", i.to_string())
                        .with_metadata("message_id", j.to_string()),
                    );
                    thread::sleep(Duration::from_millis(100));
                }
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Demonstrate error logging with context
    match perform_risky_operation() {
        Ok(_) => {
            logger.log(&Record::new(
                LogLevel::Info,
                "Operation completed successfully",
                None,
                None,
                None,
            ));
        }
        Err(e) => {
            logger.log(
                &Record::new(LogLevel::Error, "Operation failed", None, None, None)
                    .with_metadata("error", &e)
                    .with_metadata("operation_id", "12345")
                    .with_metadata("attempt", "3"),
            );
        }
    }

    // Demonstrate log level filtering
    logger.set_level(LogLevel::Warning);
    logger.log(&Record::new(
        LogLevel::Debug,
        "This debug message won't be logged",
        None,
        None,
        None,
    ));
    logger.log(&Record::new(
        LogLevel::Warning,
        "This warning message will be logged",
        None,
        None,
        None,
    ));

    // Demonstrate handler-specific filtering
    // Note: We can't modify individual handlers after initialization
    logger.log(&Record::new(
        LogLevel::Info,
        "This info message won't go to console but will go to file",
        None,
        None,
        None,
    ));

    // Basic scope usage
    let result = rust_loguru::scope::with_scope("outer", ScopeType::Regular, || {
        info!("Inside outer scope");

        // Nested scope
        let inner_result = rust_loguru::scope::with_scope("inner", ScopeType::Regular, || {
            debug!("Inside inner scope");
            thread::sleep(Duration::from_millis(100));
            42
        });

        inner_result * 2
    });

    println!("Result: {}", result);

    // Critical scope with error handling
    let _ = rust_loguru::scope::with_scope("critical_operation", ScopeType::Critical, || {
        info!("Starting critical operation");
        if false {
            // Simulated error condition
            error!("Critical operation failed");
            return Err("Operation failed");
        }
        Ok(())
    });

    // Profiling scope
    let _ = rust_loguru::scope::with_scope("performance_critical", ScopeType::Profiling, || {
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
    let _ = rust_loguru::scope::with_scope("resource_intensive", ScopeType::Resource, || {
        let mut guard = ScopeGuard::enter("file_processing", ScopeType::Resource);

        // Simulate file processing
        thread::sleep(Duration::from_millis(200));

        // Update resource usage
        let mut metrics = rust_loguru::scope::ResourceMetrics::default();
        metrics.memory_usage = 2 * 1024 * 1024; // 2MB
        metrics.io_operations = 100;
        guard.update_metrics(metrics);
    });
}

fn perform_risky_operation() -> Result<(), String> {
    // Simulate a risky operation that might fail
    if random() {
        Ok(())
    } else {
        Err("Operation failed due to random chance".to_string())
    }
}
