# Rust-Loguru

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/rust-loguru.svg)](https://crates.io/crates/rust-loguru)
[![Documentation](https://docs.rs/rust-loguru/badge.svg)](https://docs.rs/rust-loguru)

A flexible and efficient logging library for Rust inspired by Python's Loguru. Designed to provide an intuitive, powerful logging experience while maintaining Rust's performance characteristics.

## Features

- **Multiple log levels**: TRACE, DEBUG, INFO, SUCCESS, WARNING, ERROR, CRITICAL
- **Thread-safe global logger**: Safe to use in multi-threaded applications
- **Extensible handler system**: Console, file, and custom handlers
- **Configurable log formatting**: Customize how log messages are displayed
- **Support for metadata in log records**: Add structured data to log messages
- **Convenient logging macros**: Easy to use macros for all log levels
- **File rotation**: Automatic file rotation based on size with retention policies
- **Colorized output**: Colorful console output for better readability
- **Source location capture**: Automatically capture file, line, and module information
- **Ergonomic error handling and context helpers**: Extension traits, error chain, panic hook, and macros

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rust-loguru = "0.1.8" # Or Newer version
```

## Quick Start

```rust
use rust_loguru::{info, debug, error, init, LogLevel, Logger};
use rust_loguru::handler::console::ConsoleHandler;
use std::sync::Arc;
use parking_lot::RwLock;

fn main() {
    // Initialize the global logger with a console handler
    let handler = Arc::new(RwLock::new(
        ConsoleHandler::stderr(LogLevel::Debug)
            .with_colors(true)
    ));
    
    let mut logger = Logger::new(LogLevel::Debug);
    logger.add_handler(handler);
    
    // Set the global logger (required for macros to work)
    init(logger);
    
    // Log messages at different levels
    debug!("This is a debug message");
    info!("This is an info message");
    error!("This is an error message: {}", "something went wrong");
    
    // Log with metadata
    use rust_loguru::log_with_metadata;
    log_with_metadata!(LogLevel::Info, "user_id" => "123", "session" => "abc123"; 
                       "User logged in: {}", "johndoe");
}
```

## Core Concepts

### Log Levels

Rust-Loguru supports the following log levels, in order of increasing severity:

- **TRACE**: Very detailed information, typically only useful for debugging specific issues
- **DEBUG**: Detailed information useful for debugging
- **INFO**: General information about the application's operation
- **SUCCESS**: Successful operations (similar to INFO but indicates success)
- **WARNING**: Potential issues that don't prevent the application from working
- **ERROR**: Errors that prevent specific operations from working
- **CRITICAL**: Critical errors that may lead to application failure

### Handlers

Handlers determine where log messages are sent. Rust-Loguru comes with:

- **ConsoleHandler**: Outputs logs to stdout or stderr
- **FileHandler**: Writes logs to a file with optional rotation
- **NullHandler**: Discards all logs (useful for testing)

#### Creating a Console Handler

```rust
use rust_loguru::handler::console::ConsoleHandler;
use rust_loguru::LogLevel;

// Output to stderr with INFO level
let handler = ConsoleHandler::stderr(LogLevel::Info)
    .with_colors(true)
    .with_pattern("{time} {level} [{file}:{line}] {message}");
```

#### Creating a File Handler

```rust
use rust_loguru::handler::file::FileHandler;
use rust_loguru::LogLevel;
use std::path::Path;

// Write to a file with rotation at 10MB and keep 5 old files
let handler = FileHandler::new(Path::new("app.log")).unwrap()
    .with_rotation(10 * 1024 * 1024)
    .with_retention(5)
    .with_colors(false);
```

### Logging with Metadata

You can add structured metadata to log records:

```rust
use rust_loguru::{Record, LogLevel, log};

// Using the Record API directly
let record = Record::new(
    LogLevel::Info,
    "User logged in",
    Some("auth_module".to_string()),
    Some("auth.rs".to_string()),
    Some(42),
)
.with_metadata("user_id", "123")
.with_metadata("ip_address", "192.168.1.1");

log(&record);

// Or using the macro
log_with_metadata!(LogLevel::Info, 
    "user_id" => "123", 
    "ip_address" => "192.168.1.1"; 
    "User logged in: {}", "johndoe"
);
```

### Customizing Formatters

Customize how logs are formatted:

```rust
use rust_loguru::formatter::Formatter;

let formatter = Formatter::new()
    .with_colors(true)
    .with_timestamp(true)
    .with_level(true)
    .with_module(true)
    .with_location(true)
    .with_pattern("{time} {level} [{file}:{line}] {message}");
```

### Configuration Presets

Use built-in configuration presets:

```rust
use rust_loguru::LoggerConfig;

// Development configuration with detailed logging
let config = LoggerConfig::development();

// Production configuration with minimal logging
let config = LoggerConfig::production();
```

### Scopes and Timed Execution

Rust-Loguru provides a `ScopeGuard` utility for measuring the duration of code blocks and managing indentation for nested scopes. This is useful for profiling, debugging, and structured logging of code execution.

```rust
use rust_loguru::ScopeGuard;
use std::thread;
use std::time::Duration;

fn main() {
    // Enter a scope and measure its duration
    let scope = ScopeGuard::enter("outer");
    println!("Indent level: {}", ScopeGuard::indent_level());
    thread::sleep(Duration::from_millis(50));
    {
        let _inner = ScopeGuard::enter("inner");
        println!("Indent level: {}", ScopeGuard::indent_level());
        thread::sleep(Duration::from_millis(20));
        // _inner dropped here, indentation decreases
    }
    println!("Indent level: {}", ScopeGuard::indent_level());
    println!("Elapsed in outer: {:?}", scope.elapsed());
    // scope dropped here
}
```

- Indentation is managed per-thread and resets after panics.
- Use `ScopeGuard::elapsed()` to get the time spent in a scope.
- Indentation increases with nested scopes and decreases on exit.

## Error Handling Utilities

Rust-Loguru provides ergonomic helpers for error handling, context, and panic reporting.

### Extension Traits for `Result` and `Option`

```rust
use rust_loguru::{ResultExt, OptionExt};

fn might_fail() -> Result<(), std::io::Error> {
    Err(std::io::Error::new(std::io::ErrorKind::Other, "fail"))
}

fn main() {
    // Log an error if it occurs
    might_fail().log_error("Failed to do something important");

    // Add context to errors
    let res = might_fail().with_context(|| "while reading config file");
    if let Err(e) = res {
        eprintln!("Error with context: {}", e);
    }

    // Log if an Option is None
    let value: Option<u32> = None;
    value.log_none("Expected a value but got None");
}
```

### Error Chain Extraction

```rust
use rust_loguru::{ResultExt, error_chain};

fn main() {
    let res: Result<(), std::io::Error> = Err(std::io::Error::new(std::io::ErrorKind::Other, "fail"));
    let res = res.with_context(|| "while doing something");
    if let Err(e) = res {
        for cause in error_chain(&e) {
            eprintln!("Cause: {}", cause);
        }
    }
}
```

### Panic Hook Installation

```rust
use rust_loguru::install_panic_hook;

fn main() {
    install_panic_hook(); // Log all panics with file/line info
    // ... rest of your code ...
}
```

### Source Location and Error Logging Macros

```rust
use rust_loguru::{source_location, log_error_with_location};

fn main() {
    let err = "Something went wrong";
    log_error_with_location!(err);
    log_error_with_location!(err, "while processing request");
}
```

## Advanced Usage

### Creating Custom Handlers

Implement the `Handler` trait to create custom handlers:

```rust
use rust_loguru::handler::Handler;
use rust_loguru::level::LogLevel;
use rust_loguru::record::Record;
use rust_loguru::formatter::Formatter;

#[derive(Debug)]
struct CustomHandler {
    level: LogLevel,
    enabled: bool,
    formatter: Formatter,
}

impl Handler for CustomHandler {
    fn level(&self) -> LogLevel {
        self.level
    }

    fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn formatter(&self) -> &Formatter {
        &self.formatter
    }

    fn set_formatter(&mut self, formatter: Formatter) {
        self.formatter = formatter;
    }

    fn handle(&mut self, record: &Record) -> bool {
        if !self.enabled || record.level() < self.level {
            return false;
        }
        
        let formatted = self.formatter.format(record);
        // Custom handling logic here
        println!("Custom handler: {}", formatted);
        true
    }
}
```

### Structured Logging with JSON

Use the structured data capability:

```rust
use rust_loguru::{Record, LogLevel, log};
use serde_json::json;

let record = Record::new(
    LogLevel::Info,
    "API request completed",
    Some("api_module".to_string()),
    Some("api.rs".to_string()),
    Some(120),
)
.with_structured_data("request", &json!({
    "method": "GET",
    "path": "/users",
    "status": 200,
    "duration_ms": 42
}))
.unwrap();

log(&record);
```

## Advanced Context Usage

Rust-Loguru supports structured, thread-local, and async-propagatable context for logs. This allows you to automatically attach metadata (like user IDs, request IDs, etc.) to every log line in a given scope or async task.

```rust
use rust_loguru::{info, debug, context};

fn main() {
    // Add context for the current thread (e.g., user ID)
    let mut ctx = context::ContextMap::new();
    ctx.insert("user_id".to_string(), context::ContextValue::String("alice".to_string()));
    context::push_context(ctx);

    // Log a message; context will be attached if integrated into Record/formatters
    info!("User logged in");

    // Add more context (e.g., request ID) in a nested scope
    let mut req_ctx = context::ContextMap::new();
    req_ctx.insert("request_id".to_string(), context::ContextValue::String("req-123".to_string()));
    context::push_context(req_ctx);

    debug!("Processing request");

    // Pop request context when done
    context::pop_context();

    // Pop user context at the end
    context::pop_context();
}
```

#### Async Context Propagation

You can propagate context across async tasks:

```rust
use rust_loguru::context;
use std::thread;

let mut ctx = context::ContextMap::new();
ctx.insert("trace_id".to_string(), context::ContextValue::String("abc123".to_string()));
context::push_context(ctx);
let arc_ctx = context::propagate_context_for_async();

thread::spawn(move || {
    context::set_context_from_arc(arc_ctx);
    // ... logging here will see the propagated context ...
});
```

## Performance Considerations

- Log records below the configured level are filtered out early for minimal overhead
- File handlers use buffered I/O for efficient disk operations
- Consider using appropriate log levels in production to minimize overhead

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under MIT of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
```