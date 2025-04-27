//! A flexible and efficient logging library for Rust.
//!
//! This library provides a powerful logging system with the following features:
//! - Multiple log levels (TRACE, DEBUG, INFO, WARNING, ERROR, CRITICAL)
//! - Thread-safe global logger
//! - Extensible handler system
//! - Configurable log formatting
//! - Support for metadata in log records
//! - Convenient logging macros
//! - Asynchronous logging with worker thread pool
//! - Log crate compatibility
//! - Compile-time filtering optimizations
//!
//! # Examples
//!
//! ```rust,no_run
//! use rust_loguru::{Logger, LogLevel, Record};
//! use rust_loguru::handler::NullHandler;
//! use rust_loguru::{info, debug, error};
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
//!
//! // Or use the convenient macros
//! info!("Hello, world!");
//! debug!("Debug message: {}", 42);
//! error!("Error occurred: {}", "something went wrong");
//!
//! // Asynchronous logging
//! logger.set_async(true, Some(10000));
//! info!("This will be logged asynchronously");
//! ```

pub mod config;
pub mod context;
pub mod error;
pub mod formatter;
pub mod formatters;
pub mod handler;
pub mod integration;
pub mod level;
pub mod logger;
#[doc(hidden)]
pub mod macros;
pub mod record;
pub mod scope;
pub mod test_utils;

pub use config::{LoggerConfig, LoggerConfigBuilder};
pub use error::{error_chain, install_panic_hook, ContextError, OptionExt, ResultExt};
pub use formatters::json::JsonFormatter;
pub use formatters::template::TemplateFormatter;
pub use formatters::text::TextFormatter;
pub use formatters::FormatterTrait;
pub use handler::Handler;
pub use level::LogLevel;
pub use logger::{global, init, log, Logger};
pub use record::Record;
pub use scope::{ScopeError, ScopeGuard};

// Re-export log crate types for compatibility
pub use log::{LevelFilter, Log, Metadata, Record as LogRecord};

// Asynchronous logging types
use crossbeam_channel::{bounded, Receiver, Sender};
use parking_lot::RwLock as PLRwLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Compile-time static log level for filtering (can be overridden at build time)
#[allow(dead_code)]
pub static STATIC_LEVEL: level::LogLevel = level::LogLevel::Trace;

/// Async logging command type
#[doc(hidden)]
pub enum AsyncLogCommand {
    /// Log a record
    Log(Record),
    /// Shut down the async logger
    Shutdown,
}

/// Handle to the async logger
#[derive(Clone, Debug)]
pub struct AsyncLoggerHandle {
    /// Channel for sending commands to the async logger
    sender: Sender<AsyncLogCommand>,
    /// Flag indicating whether the async logger is running
    running: Arc<AtomicBool>,
}

impl AsyncLoggerHandle {
    /// Log a record
    pub fn log(&self, record: Record) -> bool {
        if !self.running.load(Ordering::Relaxed) {
            return false;
        }

        self.sender.try_send(AsyncLogCommand::Log(record)).is_ok()
    }

    /// Shut down the async logger
    pub fn shutdown(&self) {
        if !self.running.load(Ordering::Relaxed) {
            return;
        }

        // Send shutdown command and update running flag
        let _ = self.sender.send(AsyncLogCommand::Shutdown);
        self.running.store(false, Ordering::Relaxed);
    }
}

impl Drop for AsyncLoggerHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Builder for the async logger
pub struct AsyncLoggerBuilder {
    /// Queue size
    queue_size: usize,
    /// Handler registry
    handlers: Vec<Arc<PLRwLock<dyn Handler>>>,
    /// Log level
    level: LogLevel,
    /// Number of worker threads
    workers: usize,
}

impl AsyncLoggerBuilder {
    /// Create a new async logger builder
    pub fn new() -> Self {
        Self {
            queue_size: 10000,
            handlers: Vec::new(),
            level: LogLevel::Info,
            workers: 1,
        }
    }

    /// Set the queue size
    pub fn with_queue_size(mut self, queue_size: usize) -> Self {
        self.queue_size = queue_size;
        self
    }

    /// Set the handlers
    pub fn with_handlers(mut self, handlers: Vec<Arc<PLRwLock<dyn Handler>>>) -> Self {
        self.handlers = handlers;
        self
    }

    /// Set the log level
    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }

    /// Set the number of worker threads
    pub fn with_workers(mut self, workers: usize) -> Self {
        self.workers = workers;
        self
    }

    /// Build the async logger
    pub fn build(self) -> AsyncLoggerHandle {
        // Create a channel for sending commands to the worker thread
        let (sender, receiver) = bounded(self.queue_size);

        // Create a running flag
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        // Spawn the worker threads
        let handlers = self.handlers.clone();
        let level = self.level;

        // Create worker thread pool
        for _ in 0..self.workers {
            let receiver = receiver.clone();
            let handlers = handlers.clone();
            let running = running_clone.clone();

            thread::spawn(move || {
                Self::worker_thread(receiver, handlers, level, running);
            });
        }

        // Create the async logger handle
        AsyncLoggerHandle { sender, running }
    }

    /// Worker thread function
    fn worker_thread(
        receiver: Receiver<AsyncLogCommand>,
        handlers: Vec<Arc<PLRwLock<dyn Handler>>>,
        level: LogLevel,
        running: Arc<AtomicBool>,
    ) {
        while running.load(Ordering::Relaxed) {
            match receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(AsyncLogCommand::Log(record)) => {
                    // Process the record
                    if record.level() >= level {
                        for handler in &handlers {
                            let guard = handler.write();
                            if guard.is_enabled() && record.level() >= guard.level() {
                                let _ = guard.handle(&record);
                            }
                        }
                    }
                }
                Ok(AsyncLogCommand::Shutdown) => {
                    running.store(false, Ordering::Relaxed);
                    break;
                }
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                    // Timeout is normal, continue waiting
                    continue;
                }
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                    // Channel closed, exit
                    running.store(false, Ordering::Relaxed);
                    break;
                }
            }
        }
    }
}

impl Default for AsyncLoggerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Module for log crate compatibility
pub mod log_adapter {
    use crate::level::LogLevel;
    use crate::logger::global;
    use crate::record::Record as LoguruRecord;
    use log::{Level, Log, Metadata, Record};

    /// Adapter for the log crate
    pub struct LogAdapter;

    impl Log for LogAdapter {
        fn enabled(&self, metadata: &Metadata) -> bool {
            let level = match metadata.level() {
                Level::Error => LogLevel::Error,
                Level::Warn => LogLevel::Warning,
                Level::Info => LogLevel::Info,
                Level::Debug => LogLevel::Debug,
                Level::Trace => LogLevel::Trace,
            };

            level >= global().read().level()
        }

        fn log(&self, record: &Record) {
            if !self.enabled(record.metadata()) {
                return;
            }

            let level = match record.level() {
                Level::Error => LogLevel::Error,
                Level::Warn => LogLevel::Warning,
                Level::Info => LogLevel::Info,
                Level::Debug => LogLevel::Debug,
                Level::Trace => LogLevel::Trace,
            };

            let loguru_record = LoguruRecord::new(
                level,
                record.args().to_string(),
                record.module_path().map(|s| s.to_string()),
                record.file().map(|s| s.to_string()),
                record.line(),
            );

            let _ = global().read().log(&loguru_record);
        }

        fn flush(&self) {
            // Nothing to flush in our implementation
        }
    }

    /// Initialize the log adapter
    pub fn init() -> Result<(), log::SetLoggerError> {
        static LOGGER: LogAdapter = LogAdapter;
        log::set_logger(&LOGGER)?;
        Ok(())
    }

    /// Set the maximum log level for the log crate
    pub fn set_max_level(level: LogLevel) {
        let max_level = match level {
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warning => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Trace => log::LevelFilter::Trace,
            _ => log::LevelFilter::Off,
        };

        log::set_max_level(max_level);
    }
}

/// Module for compile-time filtering optimizations
pub mod compile_time {
    use crate::level::LogLevel;

    /// Compile-time level check
    #[macro_export]
    macro_rules! compile_time_level_enabled {
        ($level:expr) => {{
            #[cfg(feature = "max_level_off")]
            {
                false
            }
            #[cfg(not(feature = "max_level_off"))]
            {
                #[cfg(feature = "max_level_error")]
                {
                    $level >= $crate::LogLevel::Error
                }
                #[cfg(feature = "max_level_warn")]
                {
                    $level >= $crate::LogLevel::Warning
                }
                #[cfg(feature = "max_level_info")]
                {
                    $level >= $crate::LogLevel::Info
                }
                #[cfg(feature = "max_level_debug")]
                {
                    $level >= $crate::LogLevel::Debug
                }
                #[cfg(feature = "max_level_trace")]
                {
                    $level >= $crate::LogLevel::Trace
                }
                #[cfg(not(any(
                    feature = "max_level_error",
                    feature = "max_level_warn",
                    feature = "max_level_info",
                    feature = "max_level_debug",
                    feature = "max_level_trace"
                )))]
                {
                    true
                }
            }
        }};
    }

    /// Dynamic runtime level check (used when compile-time check passes)
    #[inline]
    pub fn runtime_level_enabled(level: LogLevel, current_level: LogLevel) -> bool {
        level >= current_level
    }
}

/// Module for benchmarking tools
pub mod benchmark {
    use crate::level::LogLevel;
    use crate::logger::Logger;
    use crate::record::Record;
    use std::time::Instant;

    /// Benchmark logger throughput
    pub fn measure_throughput(logger: &Logger, iterations: usize, level: LogLevel) -> f64 {
        let start = Instant::now();

        for i in 0..iterations {
            let record = Record::new(
                level,
                format!("Benchmark message {}", i),
                Some("benchmark".to_string()),
                Some("benchmark.rs".to_string()),
                Some(1),
            );
            let _ = logger.log(&record);
        }

        let elapsed = start.elapsed();
        iterations as f64 / elapsed.as_secs_f64()
    }

    /// Compare sync vs async performance
    pub fn compare_sync_vs_async(
        logger: &mut Logger,
        iterations: usize,
        level: LogLevel,
    ) -> (f64, f64) {
        // Measure sync throughput
        let sync_throughput = measure_throughput(logger, iterations, level);

        // Enable async logging
        logger.set_async(true, Some(iterations));

        // Measure async throughput
        let async_throughput = measure_throughput(logger, iterations, level);

        // Disable async logging
        logger.set_async(false, None);

        (sync_throughput, async_throughput)
    }
}
