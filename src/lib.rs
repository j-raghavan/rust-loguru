//! A flexible and efficient logging library for Rust.
//!
//! This library provides a powerful logging system with the following features:
//! - Multiple log levels (TRACE, DEBUG, INFO, WARNING, ERROR, CRITICAL)
//! - Thread-safe global logger
//! - Extensible handler system
//! - Configurable log formatting
//! - Support for metadata in log records
//!
//! # Examples
//!
//! ```rust,no_run
//! use rust_loguru::{Logger, LogLevel, Record};
//! use rust_loguru::handler::NullHandler;
//! use std::sync::Arc;
//! use parking_lot::RwLock;
//!
//! // Create a logger with a handler
//! let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
//! let mut logger = Logger::new(LogLevel::Debug);
//! logger.add_handler(handler);
//!
//! // Log a message
//! let record = Record::new(
//!     LogLevel::Info,
//!     "Hello, world!",
//!     Some("my_module".to_string()),
//!     Some("main.rs".to_string()),
//!     Some(42),
//! );
//! logger.log(&record);
//! ```

pub mod formatter;
pub mod handler;
pub mod level;
pub mod logger;
pub mod record;

pub use handler::Handler;
pub use level::LogLevel;
pub use logger::{global, init, log, Logger};
pub use record::Record;
