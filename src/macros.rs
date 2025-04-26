//! Macros for convenient logging.
//!
//! This module provides macros for each log level that make it easy to log messages
//! with source location information and formatting support.

/// Logs a message at the TRACE level.
///
/// # Examples
///
/// ```rust
/// use rust_loguru::trace;
///
/// trace!("This is a trace message");
/// trace!("Formatted message: {}", 42);
/// ```
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        $crate::log(&$crate::Record::new(
            $crate::LogLevel::Trace,
            format!($($arg)*),
            Some(module_path!().to_string()),
            Some(file!().to_string()),
            Some(line!()),
        ))
    };
}

/// Logs a message at the DEBUG level.
///
/// # Examples
///
/// ```rust
/// use rust_loguru::debug;
///
/// debug!("This is a debug message");
/// debug!("Formatted message: {}", 42);
/// ```
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::log(&$crate::Record::new(
            $crate::LogLevel::Debug,
            format!($($arg)*),
            Some(module_path!().to_string()),
            Some(file!().to_string()),
            Some(line!()),
        ))
    };
}

/// Logs a message at the INFO level.
///
/// # Examples
///
/// ```rust
/// use rust_loguru::info;
///
/// info!("This is an info message");
/// info!("Formatted message: {}", 42);
/// ```
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::log(&$crate::Record::new(
            $crate::LogLevel::Info,
            format!($($arg)*),
            Some(module_path!().to_string()),
            Some(file!().to_string()),
            Some(line!()),
        ))
    };
}

/// Logs a message at the SUCCESS level.
///
/// # Examples
///
/// ```rust
/// use rust_loguru::success;
///
/// success!("This is a success message");
/// success!("Formatted message: {}", 42);
/// ```
#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {
        $crate::log(&$crate::Record::new(
            $crate::LogLevel::Success,
            format!($($arg)*),
            Some(module_path!().to_string()),
            Some(file!().to_string()),
            Some(line!()),
        ))
    };
}

/// Logs a message at the WARNING level.
///
/// # Examples
///
/// ```rust
/// use rust_loguru::warn;
///
/// warn!("This is a warning message");
/// warn!("Formatted message: {}", 42);
/// ```
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::log(&$crate::Record::new(
            $crate::LogLevel::Warning,
            format!($($arg)*),
            Some(module_path!().to_string()),
            Some(file!().to_string()),
            Some(line!()),
        ))
    };
}

/// Logs a message at the ERROR level.
///
/// # Examples
///
/// ```rust
/// use rust_loguru::error;
///
/// error!("This is an error message");
/// error!("Formatted message: {}", 42);
/// ```
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::log(&$crate::Record::new(
            $crate::LogLevel::Error,
            format!($($arg)*),
            Some(module_path!().to_string()),
            Some(file!().to_string()),
            Some(line!()),
        ))
    };
}

/// Logs a message at the CRITICAL level.
///
/// # Examples
///
/// ```rust
/// use rust_loguru::critical;
///
/// critical!("This is a critical message");
/// critical!("Formatted message: {}", 42);
/// ```
#[macro_export]
macro_rules! critical {
    ($($arg:tt)*) => {
        $crate::log(&$crate::Record::new(
            $crate::LogLevel::Critical,
            format!($($arg)*),
            Some(module_path!().to_string()),
            Some(file!().to_string()),
            Some(line!()),
        ))
    };
}

/// Logs a message with metadata at the specified level.
///
/// # Examples
///
/// ```rust
/// use rust_loguru::{log_with_metadata, LogLevel};
///
/// log_with_metadata!(LogLevel::Info, "key" => "value"; "This is a message");
/// log_with_metadata!(LogLevel::Error, "error_code" => "E123"; "Failed to process: {}", 42);
/// ```
#[macro_export]
macro_rules! log_with_metadata {
    ($level:expr, $($key:expr => $value:expr),+; $($arg:tt)*) => {
        {
            let mut record = $crate::Record::new(
                $level,
                format!($($arg)*),
                Some(module_path!().to_string()),
                Some(file!().to_string()),
                Some(line!()),
            );
            $(
                record = record.with_metadata($key, $value);
            )+
            let result = $crate::log(&record);
            println!("Log with metadata result: {}", result);
            result
        }
    };
}
