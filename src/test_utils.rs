//! Test utilities for Rust-Loguru
//!
//! Provides log capture, assertion macros, and mock logger for testing.

use crate::level::LogLevel;
use crate::record::Record;
use std::sync::{Arc, Mutex};

/// Captured log record
#[derive(Debug, Clone)]
pub struct CapturedRecord {
    pub level: LogLevel,
    pub message: String,
    pub module: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
}

/// Log capture system for tests
#[derive(Clone, Default)]
pub struct LogCapture {
    records: Arc<Mutex<Vec<CapturedRecord>>>,
}

impl LogCapture {
    pub fn new() -> Self {
        Self {
            records: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn handle(&self, record: &Record) {
        let captured = CapturedRecord {
            level: record.level(),
            message: record.message().to_string(),
            module: Some(record.module().to_string()),
            file: Some(record.file().to_string()),
            line: Some(record.line()),
        };
        self.records.lock().unwrap().push(captured);
    }

    pub fn clear(&self) {
        self.records.lock().unwrap().clear();
    }

    pub fn records(&self) -> Vec<CapturedRecord> {
        self.records.lock().unwrap().clone()
    }

    pub fn contains_message(&self, msg: &str) -> bool {
        self.records().iter().any(|r| r.message.contains(msg))
    }

    pub fn contains_level(&self, level: LogLevel) -> bool {
        self.records().iter().any(|r| r.level == level)
    }
}

/// Mock logger for testing
pub struct MockLogger {
    pub capture: LogCapture,
    pub level: LogLevel,
}

impl MockLogger {
    pub fn new(level: LogLevel) -> Self {
        Self {
            capture: LogCapture::new(),
            level,
        }
    }

    pub fn log(&self, record: &Record) -> bool {
        if record.level() >= self.level {
            self.capture.handle(record);
            true
        } else {
            false
        }
    }

    pub fn level(&self) -> LogLevel {
        self.level
    }

    pub fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }
}

/// Assertion macro for log capture
#[macro_export]
macro_rules! assert_log_contains {
    ($capture:expr, $msg:expr) => {
        assert!(
            $capture.contains_message($msg),
            "Expected log to contain message: {}\nCaptured: {:?}",
            $msg,
            $capture.records()
        );
    };
}

/// Assertion macro for log level
#[macro_export]
macro_rules! assert_log_level {
    ($capture:expr, $level:expr) => {
        assert!(
            $capture.contains_level($level),
            "Expected log to contain level: {:?}\nCaptured: {:?}",
            $level,
            $capture.records()
        );
    };
}

/// Temporarily set log level for a logger in a test
pub struct TempLogLevel<'a, L: ?Sized> {
    logger: &'a mut L,
    old_level: LogLevel,
    set_level: fn(&mut L, LogLevel),
}

impl<'a, L: ?Sized> TempLogLevel<'a, L> {
    pub fn new(
        logger: &'a mut L,
        new_level: LogLevel,
        set_level: fn(&mut L, LogLevel),
        get_level: fn(&L) -> LogLevel,
    ) -> Self {
        let old_level = get_level(logger);
        set_level(logger, new_level);
        Self {
            logger,
            old_level,
            set_level,
        }
    }
}

impl<L: ?Sized> Drop for TempLogLevel<'_, L> {
    fn drop(&mut self) {
        (self.set_level)(self.logger, self.old_level);
    }
}
