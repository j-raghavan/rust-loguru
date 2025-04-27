use rust_loguru::scope::*;
use std::panic;
use std::thread;
use std::time::Duration;

#[test]
fn test_scope_timing_accuracy() {
    let guard = ScopeGuard::enter("timing");
    thread::sleep(Duration::from_millis(20));
    let elapsed = guard.elapsed();
    assert!(elapsed >= Duration::from_millis(20));
}

#[test]
fn test_scope_indentation_nested() {
    assert_eq!(ScopeGuard::indent_level(), 0);
    let _g1 = ScopeGuard::enter("outer");
    assert_eq!(ScopeGuard::indent_level(), 1);
    {
        let _g2 = ScopeGuard::enter("inner");
        assert_eq!(ScopeGuard::indent_level(), 2);
    }
    assert_eq!(ScopeGuard::indent_level(), 1);
    drop(_g1);
    assert_eq!(ScopeGuard::indent_level(), 0);
}

#[test]
fn test_scope_explicit_exit() {
    let guard = ScopeGuard::enter("explicit_exit");
    guard.exit();
    // Indentation should still be correct after drop
    assert_eq!(ScopeGuard::indent_level(), 0);
}

#[test]
fn test_scope_multiple_scopes() {
    let _g1 = ScopeGuard::enter("scope1");
    let _g2 = ScopeGuard::enter("scope2");
    let _g3 = ScopeGuard::enter("scope3");
    assert_eq!(ScopeGuard::indent_level(), 3);
    // Drop in reverse order
    drop(_g3);
    assert_eq!(ScopeGuard::indent_level(), 2);
    drop(_g2);
    assert_eq!(ScopeGuard::indent_level(), 1);
    drop(_g1);
    assert_eq!(ScopeGuard::indent_level(), 0);
}

#[test]
fn test_scope_panic_resets_indent() {
    let result = panic::catch_unwind(|| {
        let _g1 = ScopeGuard::enter("panic_outer");
        let _g2 = ScopeGuard::enter("panic_inner");
        assert_eq!(ScopeGuard::indent_level(), 2);
        panic!("panic in scope");
    });
    assert!(result.is_err());
    assert_eq!(ScopeGuard::indent_level(), 0);
}

#[test]
fn test_scope_debug_fmt() {
    let guard = ScopeGuard::enter("debug_fmt");
    let s = format!("{:?}", guard);
    assert!(s.contains("ScopeGuard"));
    assert!(s.contains("debug_fmt"));
}

#[test]
fn test_scope_zero_duration() {
    let guard = ScopeGuard::enter("zero_duration");
    let elapsed = guard.elapsed();
    // Should be very small, but non-negative
    assert!(elapsed >= Duration::from_millis(0));
}
