use rust_loguru::handler::console::ConsoleHandler;
use rust_loguru::handler::file::FileHandler;
use rust_loguru::handler::new_handler_ref;
use rust_loguru::level::LogLevel;
use rust_loguru::logger::Logger;
use rust_loguru::Record;
use rust_loguru::{install_panic_hook, OptionExt, ResultExt};

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
    let logger = rust_loguru::init(logger);

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

    // --- Error Handling Utilities Minimal Example ---
    install_panic_hook();
    let res: Result<(), std::io::Error> =
        Err(std::io::Error::new(std::io::ErrorKind::Other, "fail"));
    let _ = res.log_error("API call failed");
    let opt: Option<u32> = None;
    opt.log_none("Missing value in API response");

    // --- Integration module usage examples ---
    // These will panic if run, but show the intended API for future integration features.
    // fn integration_examples() {
    //     // use rust_loguru::integration::{async_runtime, log_compat, middleware};
    //     // Log crate compatibility
    //     // log_compat::init_loguru_as_log(); // Will panic

    //     // Async runtime integration
    //     // async_runtime::integrate_with_tokio(); // Will panic

    //     // Framework middleware
    //     // middleware::request_response_logging(); // Will panic
    // }
}
