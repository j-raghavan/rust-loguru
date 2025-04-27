//! Error handling utilities for rust-loguru
//!
//! - Extension traits for Result/Option
//! - Error chain capturing
//! - Panic hook installation
//! - Source location capture
//! - Helper methods for error logging

use std::error::Error as StdError;
use std::fmt::{self, Debug, Display};
use std::panic::{self, PanicHookInfo};
use std::sync::Once;

/// Extension trait for Result to add logging and context
pub trait ResultExt<T, E> {
    /// Log the error if it is Err, and return self
    fn log_error(self, msg: &str) -> Self;
    /// Attach context to the error
    fn with_context<F, C>(self, f: F) -> Result<T, ContextError<E, C>>
    where
        F: FnOnce() -> C,
        E: StdError + 'static,
        C: Display + Debug + Send + Sync + 'static;
}

impl<T, E> ResultExt<T, E> for Result<T, E>
where
    E: StdError + 'static,
{
    fn log_error(self, msg: &str) -> Self {
        if let Err(ref e) = self {
            eprintln!("[ERROR] {}: {}", msg, e);
        }
        self
    }

    fn with_context<F, C>(self, f: F) -> Result<T, ContextError<E, C>>
    where
        F: FnOnce() -> C,
        C: Display + Debug + Send + Sync + 'static,
    {
        self.map_err(|e| ContextError {
            error: e,
            context: f(),
        })
    }
}

/// Extension trait for Option to add logging
pub trait OptionExt<T> {
    /// Log if None, and return self
    fn log_none(self, msg: &str) -> Self;
}

impl<T> OptionExt<T> for Option<T> {
    fn log_none(self, msg: &str) -> Self {
        if self.is_none() {
            eprintln!("[ERROR] {}: None value", msg);
        }
        self
    }
}

/// Error type with context
#[derive(Debug)]
pub struct ContextError<E, C> {
    pub error: E,
    pub context: C,
}

impl<E, C> Display for ContextError<E, C>
where
    E: Display,
    C: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (context: {})", self.error, self.context)
    }
}

impl<E, C> StdError for ContextError<E, C>
where
    E: StdError + 'static,
    C: Display + Debug + Send + Sync + 'static,
{
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.error)
    }
}

/// Capture the error chain as a Vec of Strings
pub fn error_chain(mut err: &(dyn StdError + 'static)) -> Vec<String> {
    let mut chain = Vec::new();
    loop {
        chain.push(format!("{}", err));
        match err.source() {
            Some(source) => err = source,
            None => break,
        }
    }
    chain
}

static PANIC_HOOK_INIT: Once = Once::new();

/// Install a panic hook that logs panics with location and payload
pub fn install_panic_hook() {
    PANIC_HOOK_INIT.call_once(|| {
        let default_hook = panic::take_hook();
        panic::set_hook(Box::new(move |info: &PanicHookInfo| {
            let location = info
                .location()
                .map(|l| l.to_string())
                .unwrap_or_else(|| "unknown location".to_string());
            let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = info.payload().downcast_ref::<String>() {
                s.clone()
            } else {
                "Box<Any>".to_string()
            };
            eprintln!("[PANIC] at {}: {}", location, payload);
            default_hook(info);
        }));
    });
}

/// Helper to extract source location (file, line, column)
#[macro_export]
macro_rules! source_location {
    () => {
        (file!(), line!(), column!())
    };
}

/// Helper to log an error with source location
#[macro_export]
macro_rules! log_error_with_location {
    ($err:expr) => {{
        let (file, line, col) = $crate::source_location!();
        eprintln!("[ERROR] at {}:{}:{}: {}", file, line, col, $err);
    }};
    ($err:expr, $msg:expr) => {{
        let (file, line, col) = $crate::source_location!();
        eprintln!("[ERROR] at {}:{}:{}: {}: {}", file, line, col, $msg, $err);
    }};
}
