use rust_loguru::context::*;
// use std::sync::Arc;

#[test]
fn test_push_and_pop_context() {
    let mut ctx = ContextMap::new();
    ctx.insert(
        "user".to_string(),
        ContextValue::String("alice".to_string()),
    );
    push_context(ctx.clone());
    assert_eq!(
        get_context_value("user"),
        Some(ContextValue::String("alice".to_string()))
    );
    pop_context();
    assert_eq!(get_context_value("user"), None);
}

#[test]
fn test_context_stack_merging() {
    let mut ctx1 = ContextMap::new();
    ctx1.insert(
        "user".to_string(),
        ContextValue::String("alice".to_string()),
    );
    push_context(ctx1);
    let mut ctx2 = ContextMap::new();
    ctx2.insert("request_id".to_string(), ContextValue::Integer(42));
    push_context(ctx2);
    let merged = current_context();
    assert_eq!(
        merged.get("user"),
        Some(&ContextValue::String("alice".to_string()))
    );
    assert_eq!(merged.get("request_id"), Some(&ContextValue::Integer(42)));
    pop_context();
    pop_context();
}

#[test]
fn test_set_and_get_context_value() {
    let mut ctx = ContextMap::new();
    ctx.insert("foo".to_string(), ContextValue::String("bar".to_string()));
    push_context(ctx);
    set_context_value("foo", ContextValue::String("baz".to_string()));
    assert_eq!(
        get_context_value("foo"),
        Some(ContextValue::String("baz".to_string()))
    );
    pop_context();
}

#[test]
fn test_context_value_types() {
    let mut ctx = ContextMap::new();
    ctx.insert("int".to_string(), ContextValue::Integer(1));
    ctx.insert("float".to_string(), ContextValue::Float(2.5));
    ctx.insert("bool".to_string(), ContextValue::Bool(true));
    push_context(ctx);
    assert_eq!(get_context_value("int"), Some(ContextValue::Integer(1)));
    assert_eq!(get_context_value("float"), Some(ContextValue::Float(2.5)));
    assert_eq!(get_context_value("bool"), Some(ContextValue::Bool(true)));
    pop_context();
}

#[test]
fn test_async_context_propagation() {
    let mut ctx = ContextMap::new();
    ctx.insert(
        "trace_id".to_string(),
        ContextValue::String("abc123".to_string()),
    );
    push_context(ctx);
    let arc_ctx = propagate_context_for_async();
    pop_context();
    // Simulate async context restoration
    set_context_from_arc(arc_ctx.clone());
    assert_eq!(
        get_context_value("trace_id"),
        Some(ContextValue::String("abc123".to_string()))
    );
    pop_context();
}
