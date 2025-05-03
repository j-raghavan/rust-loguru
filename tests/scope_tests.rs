use rust_loguru::scope::{ResourceMetrics, ScopeGuard, ScopeType};
use std::panic;
use std::thread;
use std::time::Duration;
use std::time::Instant;

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
    let result = rust_loguru::scope::with_scope("test_with_scope", ScopeType::Regular, || {
        assert_eq!(ScopeGuard::indent_level(), 1);
        42
    });
    assert_eq!(result, 42);
    assert_eq!(ScopeGuard::indent_level(), 0);
}

#[test]
#[should_panic(expected = "test panic")]
fn test_with_scope_panic() {
    rust_loguru::scope::with_scope("panic_scope", ScopeType::Critical, || {
        panic!("test panic");
    });
}

#[test]
fn test_scope_id_uniqueness() {
    let id1 = rust_loguru::scope::with_scope("scope1", ScopeType::Regular, || {
        let guard = ScopeGuard::enter("inner1", ScopeType::Regular);
        guard.id()
    });

    let id2 = rust_loguru::scope::with_scope("scope2", ScopeType::Regular, || {
        let guard = ScopeGuard::enter("inner2", ScopeType::Regular);
        guard.id()
    });

    assert_ne!(id1, id2);
}

#[test]
fn test_profile_scope() {
    let result = rust_loguru::scope::with_scope("profiling", ScopeType::Profiling, || {
        thread::sleep(Duration::from_millis(10));
        let mut guard = ScopeGuard::enter("inner", ScopeType::Profiling);
        let mut metrics = ResourceMetrics::default();
        metrics.cpu_time = Duration::from_millis(100);
        metrics.memory_usage = 1024;
        metrics.io_operations = 5;
        guard.update_metrics(metrics);
        42
    });
    assert_eq!(result, 42);
}

#[test]
fn test_resource_scope() {
    let result = rust_loguru::scope::with_scope("resource_tracking", ScopeType::Resource, || {
        let mut guard = ScopeGuard::enter("inner", ScopeType::Resource);
        let mut metrics = ResourceMetrics::default();
        metrics.memory_usage = 2048;
        metrics.io_operations = 10;
        guard.update_metrics(metrics);
        "resource tracked"
    });
    assert_eq!(result, "resource tracked");
}

#[test]
fn test_scope_metrics() {
    rust_loguru::scope::with_scope("metrics_test", ScopeType::Profiling, || {
        let mut guard = ScopeGuard::enter("metrics", ScopeType::Profiling);
        let mut metrics = ResourceMetrics::default();
        metrics.cpu_time = Duration::from_millis(150);
        metrics.memory_usage = 4096;
        metrics.io_operations = 20;
        guard.update_metrics(metrics);

        let stored_metrics = guard.metrics();
        assert_eq!(stored_metrics.cpu_time, Duration::from_millis(150));
        assert_eq!(stored_metrics.memory_usage, 4096);
        assert_eq!(stored_metrics.io_operations, 20);
    });
}
