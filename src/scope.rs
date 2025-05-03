//! Scope management for rust-loguru
//!
//! - Timed execution scopes
//! - Indentation management
//! - Enter/exit tracking
//! - Performance timing
//! - Resource tracking
//! - Specialized scope types

use std::cell::Cell;
use std::fmt;
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use crate::{debug, error, info, trace};

// Thread-local indentation level for scopes
thread_local! {
    static INDENT_LEVEL: Cell<usize> = const { Cell::new(0) };
}

// Global scope ID counter
static SCOPE_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Error type for scope operations
#[derive(Debug, Clone, PartialEq)]
pub struct ScopeError(pub &'static str);

/// Type of scope for specialized behavior
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScopeType {
    /// Regular scope with timing
    Regular,
    /// Critical scope with enhanced error handling
    Critical,
    /// Profiling scope with detailed metrics
    Profiling,
    /// Resource tracking scope
    Resource,
}

/// Resource metrics tracked by scopes
#[derive(Debug, Clone, Default)]
pub struct ResourceMetrics {
    pub cpu_time: Duration,
    pub memory_usage: usize,
    pub io_operations: usize,
}

/// A guard that tracks the lifetime of a scope, measures timing, and manages indentation
pub struct ScopeGuard {
    id: usize,
    name: &'static str,
    scope_type: ScopeType,
    start: Instant,
    indent: usize,
    exited: bool,
    metrics: ResourceMetrics,
    panic_info: Option<String>,
}

impl ScopeGuard {
    /// Enter a new scope with the given name and type
    pub fn enter(name: &'static str, scope_type: ScopeType) -> Self {
        let id = SCOPE_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let indent = INDENT_LEVEL.with(|lvl| {
            let current = lvl.get();
            lvl.set(current + 1);
            current + 1
        });
        let start = Instant::now();

        // Log scope entry based on type
        match scope_type {
            ScopeType::Critical => {
                info!("Entering critical scope: {}", name);
            }
            ScopeType::Profiling => {
                debug!("Entering profiling scope: {}", name);
            }
            ScopeType::Resource => {
                debug!("Entering resource scope: {}", name);
            }
            _ => {
                trace!("Entering scope: {}", name);
            }
        }

        ScopeGuard {
            id,
            name,
            scope_type,
            start,
            indent,
            exited: false,
            metrics: ResourceMetrics::default(),
            panic_info: None,
        }
    }

    /// Get the elapsed time since entering the scope
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Get the current indentation level
    pub fn indent_level() -> usize {
        INDENT_LEVEL.with(|lvl| lvl.get())
    }

    /// Get the scope's unique ID
    pub fn id(&self) -> usize {
        self.id
    }

    /// Get the scope's metrics
    pub fn metrics(&self) -> &ResourceMetrics {
        &self.metrics
    }

    /// Update resource metrics
    pub fn update_metrics(&mut self, metrics: ResourceMetrics) {
        self.metrics = metrics;
    }

    /// Explicitly exit the scope (optional, usually handled by Drop)
    pub fn exit(mut self) {
        self.exited = true;
        self.log_exit();
    }

    /// Log scope exit with appropriate information
    fn log_exit(&self) {
        let elapsed = self.elapsed();
        match self.scope_type {
            ScopeType::Critical => {
                if let Some(ref panic_info) = self.panic_info {
                    error!(
                        "Critical scope '{}' exited with panic: {}",
                        self.name, panic_info
                    );
                } else {
                    info!("Critical scope '{}' completed in {:?}", self.name, elapsed);
                }
            }
            ScopeType::Profiling => {
                debug!(
                    "Profiling scope '{}' completed in {:?} (CPU: {:?}, Memory: {} bytes, I/O: {})",
                    self.name,
                    elapsed,
                    self.metrics.cpu_time,
                    self.metrics.memory_usage,
                    self.metrics.io_operations
                );
            }
            ScopeType::Resource => {
                debug!(
                    "Resource scope '{}' completed in {:?} (Memory: {} bytes, I/O: {})",
                    self.name, elapsed, self.metrics.memory_usage, self.metrics.io_operations
                );
            }
            _ => {
                trace!("Scope '{}' completed in {:?}", self.name, elapsed);
            }
        }
    }
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        // Decrement indentation
        INDENT_LEVEL.with(|lvl| {
            let current = lvl.get();
            if current > 0 {
                lvl.set(current - 1);
            }
        });

        // If not already exited, log the exit
        if !self.exited {
            self.log_exit();
        }
    }
}

impl fmt::Debug for ScopeGuard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScopeGuard")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("type", &self.scope_type)
            .field("indent", &self.indent)
            .field("elapsed", &self.elapsed())
            .field("metrics", &self.metrics)
            .field("panic_info", &self.panic_info)
            .finish()
    }
}

/// Execute a block of code within a scope
pub fn with_scope<F, R>(name: &'static str, scope_type: ScopeType, f: F) -> R
where
    F: FnOnce() -> R,
{
    let guard = ScopeGuard::enter(name, scope_type);
    let result = panic::catch_unwind(AssertUnwindSafe(f));

    match result {
        Ok(r) => r,
        Err(e) => {
            let panic_info = if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "Unknown panic".to_string()
            };

            // Store panic info in guard
            let mut guard = guard;
            guard.panic_info = Some(panic_info.clone());

            // For critical scopes, we might want to handle the panic differently
            if scope_type == ScopeType::Critical {
                error!("Critical scope '{}' panicked", name);
            }

            panic::resume_unwind(e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_scope_basic_timing() {
        let start = Instant::now();
        let guard = ScopeGuard::enter("test_scope", ScopeType::Regular);
        thread::sleep(Duration::from_millis(10));
        let elapsed = guard.elapsed();
        assert!(elapsed >= Duration::from_millis(10));
        drop(guard);
        let total = start.elapsed();
        assert!(total >= Duration::from_millis(10));
    }

    #[test]
    fn test_scope_indentation() {
        assert_eq!(ScopeGuard::indent_level(), 0);
        {
            let _g1 = ScopeGuard::enter("outer", ScopeType::Regular);
            assert_eq!(ScopeGuard::indent_level(), 1);
            {
                let _g2 = ScopeGuard::enter("inner", ScopeType::Regular);
                assert_eq!(ScopeGuard::indent_level(), 2);
            }
            assert_eq!(ScopeGuard::indent_level(), 1);
        }
        assert_eq!(ScopeGuard::indent_level(), 0);
    }

    #[test]
    fn test_scope_exit_explicit() {
        let guard = ScopeGuard::enter("explicit_exit", ScopeType::Regular);
        guard.exit();
        assert_eq!(ScopeGuard::indent_level(), 0);
    }

    #[test]
    fn test_scope_panic_handling() {
        let result = panic::catch_unwind(|| {
            let _guard = ScopeGuard::enter("panic_scope", ScopeType::Critical);
            panic!("test panic");
        });
        assert!(result.is_err());
        assert_eq!(ScopeGuard::indent_level(), 0);
    }

    #[test]
    fn test_resource_metrics() {
        let mut guard = ScopeGuard::enter("resource_scope", ScopeType::Resource);
        let metrics = ResourceMetrics {
            cpu_time: Duration::from_millis(100),
            memory_usage: 1024,
            io_operations: 5,
        };
        guard.update_metrics(metrics);
        assert_eq!(guard.metrics().memory_usage, 1024);
        assert_eq!(guard.metrics().io_operations, 5);
    }

    #[test]
    fn test_with_scope() {
        let result = with_scope("test_with_scope", ScopeType::Regular, || {
            assert_eq!(ScopeGuard::indent_level(), 1);
            42
        });
        assert_eq!(result, 42);
        assert_eq!(ScopeGuard::indent_level(), 0);
    }

    #[test]
    #[should_panic(expected = "test panic")]
    fn test_with_scope_panic() {
        with_scope("panic_scope", ScopeType::Critical, || {
            panic!("test panic");
        });
    }
}
