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
    let snapshot = create_context_snapshot();
    pop_context();
    // Simulate async context restoration
    restore_context(&snapshot);
    assert_eq!(
        get_context_value("trace_id"),
        Some(ContextValue::String("abc123".to_string()))
    );
    pop_context();
}

#[test]
fn test_global_context() {
    set_global_context_value("app_id", ContextValue::String("test-app".to_string()));
    assert_eq!(
        get_global_context_value("app_id"),
        Some(ContextValue::String("test-app".to_string()))
    );
}

#[test]
fn test_context_scope() {
    {
        let _scope = create_context_scope();
        set_context_value("request_id", ContextValue::String("123".to_string()));
        assert_eq!(
            get_context_value("request_id"),
            Some(ContextValue::String("123".to_string()))
        );
    }
    // Context should be automatically cleared after scope ends
    assert_eq!(get_context_value("request_id"), None);
}

#[test]
fn test_nested_context_scopes() {
    {
        let _scope1 = create_context_scope();
        set_context_value("outer", ContextValue::String("outer_value".to_string()));
        {
            let _scope2 = create_context_scope();
            set_context_value("inner", ContextValue::String("inner_value".to_string()));
            assert_eq!(
                get_context_value("outer"),
                Some(ContextValue::String("outer_value".to_string()))
            );
            assert_eq!(
                get_context_value("inner"),
                Some(ContextValue::String("inner_value".to_string()))
            );
        }
        assert_eq!(
            get_context_value("outer"),
            Some(ContextValue::String("outer_value".to_string()))
        );
        assert_eq!(get_context_value("inner"), None);
    }
    assert_eq!(get_context_value("outer"), None);
    assert_eq!(get_context_value("inner"), None);
}
