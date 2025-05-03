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

// --- Context Macros ---
/// Push a new context map onto the stack.
/// Usage: push_context! { "key1" => val1, "key2" => val2 }
#[macro_export]
macro_rules! push_context {
    ( $( $key:expr => $val:expr ),* $(,)? ) => {
        {
            let mut ctx = ::std::collections::HashMap::new();
            $( ctx.insert($key.to_string(), $crate::context::ContextValue::String($val.to_string())); )*
            $crate::context::push_context(ctx);
        }
    };
}

/// Pop the top context map from the stack.
#[macro_export]
macro_rules! pop_context {
    () => {
        $crate::context::pop_context();
    };
}

/// Set a key-value pair in the current context.
#[macro_export]
macro_rules! set_context {
    ($key:expr, $val:expr) => {
        $crate::context::set_context_value(
            $key,
            $crate::context::ContextValue::String($val.to_string()),
        );
    };
}

/// Get a value from the current context.
#[macro_export]
macro_rules! get_context {
    ($key:expr) => {
        $crate::context::get_context_value($key)
    };
}

/// Create a new context scope with automatic cleanup.
/// Usage: with_context! { "key" => "value" => { /* code */ } }
#[macro_export]
macro_rules! with_context {
    ( $( $key:expr => $val:expr ),* $(,)? => $block:block ) => {
        {
            let _guard = $crate::context::create_context_scope();
            $(
                $crate::context::set_context_value(
                    $key,
                    $crate::context::ContextValue::String($val.to_string()),
                );
            )*
            $block
        }
    };
}

/// Create a new async context scope.
/// Usage: async_with_context! { "key" => "value" => async { /* code */ } }
#[macro_export]
macro_rules! async_with_context {
    ( $( $key:expr => $val:expr ),* $(,)? => $block:block ) => {
        {
            let snapshot = $crate::context::create_context_snapshot();
            $(
                $crate::context::set_context_value(
                    $key,
                    $crate::context::ContextValue::String($val.to_string()),
                );
            )*
            async move {
                let _guard = $crate::context::create_context_scope();
                $crate::context::restore_context(&snapshot);
                $block
            }
        }
    };
}

/// Set a value in the global context.
#[macro_export]
macro_rules! set_global_context {
    ($key:expr, $val:expr) => {
        $crate::context::set_global_context_value(
            $key,
            $crate::context::ContextValue::String($val.to_string()),
        );
    };
}

/// Get a value from the global context.
#[macro_export]
macro_rules! get_global_context {
    ($key:expr) => {
        $crate::context::get_global_context_value($key)
    };
}

/// Create a new context scope with automatic cleanup and return value.
/// Usage: let result = context_scope! { "key" => "value" => { /* code */ } }
#[macro_export]
macro_rules! context_scope {
    ( $( $key:expr => $val:expr ),* $(,)? => $block:block ) => {
        {
            let _guard = $crate::context::create_context_scope();
            $(
                $crate::context::set_context_value(
                    $key,
                    $crate::context::ContextValue::String($val.to_string()),
                );
            )*
            $block
        }
    };
}

// --- Scope Macros ---
/// Enter a new scope. Returns a guard that times and manages indentation.
/// Usage: let _scope = scope!("my_scope");
#[macro_export]
macro_rules! scope {
    ($name:expr) => {
        $crate::scope::ScopeGuard::enter($name)
    };
}

/// Log entering and exiting a scope at INFO level, with timing.
/// Usage: let _scope = scoped_info!("my_scope");
#[macro_export]
macro_rules! scoped_info {
    ($name:expr) => {{
        $crate::info!("Entering scope: {}", $name);
        let _guard = $crate::scope::ScopeGuard::enter($name);
        struct ScopeLogger<'a> {
            name: &'a str,
            start: ::std::time::Instant,
        }
        impl<'a> ::std::ops::Drop for ScopeLogger<'a> {
            fn drop(&mut self) {
                let elapsed = self.start.elapsed();
                $crate::info!("Exiting scope: {} (elapsed: {:?})", self.name, elapsed);
            }
        }
        ScopeLogger {
            name: $name,
            start: ::std::time::Instant::now(),
        }
    }};
}

// --- Error Integration Macros ---
/// Log an error with source location (uses error! macro)
#[macro_export]
macro_rules! log_error {
    ($err:expr) => {
        $crate::error!("{}", $err);
    };
    ($err:expr, $msg:expr) => {
        $crate::error!("{}: {}", $msg, $err);
    };
}

/// Log an error with context (uses error! macro)
#[macro_export]
macro_rules! log_error_with_context {
    ($err:expr, $ctx:expr) => {
        $crate::error!("{} (context: {:?})", $err, $ctx);
    };
}

/// Try to log an error if Result is Err or Option is None, then return the value.
#[macro_export]
macro_rules! try_log {
    ($expr:expr, $msg:expr) => {{
        match $expr {
            Ok(val) => Ok(val),
            Err(e) => {
                $crate::error!("{}: {}", $msg, e);
                Err(e)
            }
        }
    }};
    (option $expr:expr, $msg:expr) => {{
        match $expr {
            Some(val) => Some(val),
            None => {
                $crate::error!("{}: None value", $msg);
                None
            }
        }
    }};
}

// --- Compile-time Level Filtering ---
// Usage: static LEVEL: LogLevel = LogLevel::Info; (in lib.rs or build.rs)
// Macros will only emit code if $level >= STATIC_LEVEL
#[macro_export]
macro_rules! log_if_enabled {
    ($level:expr, $($arg:tt)*) => {
        if $level >= $crate::STATIC_LEVEL {
            $crate::log(&$crate::Record::new(
                $level,
                format!($($arg)*),
                Some(module_path!().to_string()),
                Some(file!().to_string()),
                Some(line!()),
            ))
        } else {
            false
        }
    };
}

// --- Structured Data in Level Macros ---
// Usage: info!("msg"; "key1" => val1, "key2" => val2)
#[macro_export]
macro_rules! info_kv {
    ($msg:expr; $( $key:expr => $val:expr ),+ ) => {{
        let mut record = $crate::Record::new(
            $crate::LogLevel::Info,
            $msg.to_string(),
            Some(module_path!().to_string()),
            Some(file!().to_string()),
            Some(line!()),
        );
        $( record = record.with_metadata($key, $val); )+
        $crate::log(&record)
    }};
}
