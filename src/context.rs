//! Context management for rust-loguru
//!
//! - Thread-local storage for context data
//! - Context stack management
//! - Structured data for context values
//! - Async propagation helpers

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

/// Type alias for context key-value pairs
pub type ContextMap = HashMap<String, ContextValue>;

/// Structured value for context
#[derive(Clone, Debug, PartialEq)]
pub enum ContextValue {
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    // Add more types as needed
}

impl std::fmt::Display for ContextValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContextValue::String(s) => write!(f, "{}", s),
            ContextValue::Integer(i) => write!(f, "{}", i),
            ContextValue::Float(fl) => write!(f, "{}", fl),
            ContextValue::Bool(b) => write!(f, "{}", b),
        }
    }
}

thread_local! {
    static CONTEXT_STACK: RefCell<Vec<ContextMap>> = const { RefCell::new(vec![]) };
}

/// Push a new context map onto the stack
pub fn push_context(ctx: ContextMap) {
    CONTEXT_STACK.with(|stack| stack.borrow_mut().push(ctx));
}

/// Pop the top context map from the stack
pub fn pop_context() {
    CONTEXT_STACK.with(|stack| {
        stack.borrow_mut().pop();
    });
}

/// Get the current merged context (top to bottom)
pub fn current_context() -> ContextMap {
    CONTEXT_STACK.with(|stack| {
        let mut merged = ContextMap::new();
        for ctx in stack.borrow().iter() {
            for (k, v) in ctx.iter() {
                merged.insert(k.clone(), v.clone());
            }
        }
        merged
    })
}

/// Set a key-value pair in the current context
pub fn set_context_value(key: &str, value: ContextValue) {
    CONTEXT_STACK.with(|stack| {
        if let Some(top) = stack.borrow_mut().last_mut() {
            top.insert(key.to_string(), value);
        }
    });
}

/// Get a value from the current context
pub fn get_context_value(key: &str) -> Option<ContextValue> {
    CONTEXT_STACK.with(|stack| {
        for ctx in stack.borrow().iter().rev() {
            if let Some(val) = ctx.get(key) {
                return Some(val.clone());
            }
        }
        None
    })
}

/// Check if there is any context data available
pub fn has_context() -> bool {
    CONTEXT_STACK.with(|stack| !stack.borrow().is_empty())
}

// Async propagation helpers (stub)
pub fn propagate_context_for_async() -> Arc<ContextMap> {
    Arc::new(current_context())
}

pub fn set_context_from_arc(ctx: Arc<ContextMap>) {
    push_context(ctx.as_ref().clone());
}
