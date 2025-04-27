//! Scope management for rust-loguru
//!
//! - Timed execution scopes
//! - Indentation management
//! - Enter/exit tracking
//! - Performance timing

use std::cell::Cell;
use std::fmt;
use std::time::{Duration, Instant};

// Thread-local indentation level for scopes
thread_local! {
    static INDENT_LEVEL: Cell<usize> = const { Cell::new(0) };
}

/// Error type for scope operations
#[derive(Debug, Clone, PartialEq)]
pub struct ScopeError(pub &'static str);

/// A guard that tracks the lifetime of a scope, measures timing, and manages indentation
pub struct ScopeGuard {
    name: &'static str,
    start: Instant,
    indent: usize,
    exited: bool,
}

impl ScopeGuard {
    /// Enter a new scope with the given name
    pub fn enter(name: &'static str) -> Self {
        let indent = INDENT_LEVEL.with(|lvl| {
            let current = lvl.get();
            lvl.set(current + 1);
            current + 1
        });
        let start = Instant::now();
        // Optionally: log entering scope here
        ScopeGuard {
            name,
            start,
            indent,
            exited: false,
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

    /// Explicitly exit the scope (optional, usually handled by Drop)
    pub fn exit(mut self) {
        self.exited = true;
        // Optionally: log exiting scope here
        // Indentation will be handled in Drop
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
        // Optionally: log scope exit and timing
        // If not already exited, mark as exited
        if !self.exited {
            self.exited = true;
        }
    }
}

impl fmt::Debug for ScopeGuard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScopeGuard")
            .field("name", &self.name)
            .field("indent", &self.indent)
            .field("elapsed", &self.elapsed())
            .finish()
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
        let guard = ScopeGuard::enter("test_scope");
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
            let _g1 = ScopeGuard::enter("outer");
            assert_eq!(ScopeGuard::indent_level(), 1);
            {
                let _g2 = ScopeGuard::enter("inner");
                assert_eq!(ScopeGuard::indent_level(), 2);
            }
            assert_eq!(ScopeGuard::indent_level(), 1);
        }
        assert_eq!(ScopeGuard::indent_level(), 0);
    }

    #[test]
    fn test_scope_exit_explicit() {
        let guard = ScopeGuard::enter("explicit_exit");
        guard.exit();
        // Indentation should still be correct after drop
        assert_eq!(ScopeGuard::indent_level(), 0);
    }

    #[test]
    fn test_scope_panic_handling() {
        let result = panic::catch_unwind(|| {
            let _guard = ScopeGuard::enter("panic_scope");
            panic!("test panic");
        });
        assert!(result.is_err());
        // Indentation should be reset after panic
        assert_eq!(ScopeGuard::indent_level(), 0);
    }
}
