//! Context management for rust-loguru
//!
//! - Thread-local storage for context data
//! - Context stack management with proper nesting
//! - Structured data for context values
//! - Async propagation helpers
//! - Global context capabilities

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::RwLock;

/// Type alias for context key-value pairs
pub type ContextMap = HashMap<String, ContextValue>;

/// Structured value for context
#[derive(Clone, Debug, PartialEq)]
pub enum ContextValue {
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Map(ContextMap),
    Array(Vec<ContextValue>),
    Null,
}

impl std::fmt::Display for ContextValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContextValue::String(s) => write!(f, "{}", s),
            ContextValue::Integer(i) => write!(f, "{}", i),
            ContextValue::Float(fl) => write!(f, "{}", fl),
            ContextValue::Bool(b) => write!(f, "{}", b),
            ContextValue::Map(m) => write!(f, "{:?}", m),
            ContextValue::Array(a) => write!(f, "{:?}", a),
            ContextValue::Null => write!(f, "null"),
        }
    }
}

// Thread-local context stack
thread_local! {
    static CONTEXT_STACK: RefCell<Vec<ContextMap>> = const { RefCell::new(vec![]) };
}

// Global context registry
lazy_static::lazy_static! {
    static ref GLOBAL_CONTEXT: RwLock<ContextMap> = RwLock::new(ContextMap::new());
}

/// Context snapshot for async propagation
#[derive(Clone, Debug)]
pub struct ContextSnapshot {
    thread_context: ContextMap,
    global_context: ContextMap,
}

impl Default for ContextSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextSnapshot {
    pub fn new() -> Self {
        Self {
            thread_context: current_context(),
            global_context: GLOBAL_CONTEXT.read().unwrap().clone(),
        }
    }

    pub fn restore(&self) {
        push_context(self.thread_context.clone());
        for (k, v) in self.global_context.iter() {
            set_global_context_value(k, v.clone());
        }
    }
}

/// Push a new context map onto the stack
pub fn push_context(ctx: ContextMap) {
    CONTEXT_STACK.with(|stack| stack.borrow_mut().push(ctx));
}

/// Pop the top context map from the stack
pub fn pop_context() -> Option<ContextMap> {
    CONTEXT_STACK.with(|stack| stack.borrow_mut().pop())
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

/// Set a value in the global context
pub fn set_global_context_value(key: &str, value: ContextValue) {
    if let Ok(mut ctx) = GLOBAL_CONTEXT.write() {
        ctx.insert(key.to_string(), value);
    }
}

/// Get a value from the global context
pub fn get_global_context_value(key: &str) -> Option<ContextValue> {
    if let Ok(ctx) = GLOBAL_CONTEXT.read() {
        ctx.get(key).cloned()
    } else {
        None
    }
}

/// Create a context snapshot for async propagation
pub fn create_context_snapshot() -> ContextSnapshot {
    ContextSnapshot::new()
}

/// Restore context from a snapshot
pub fn restore_context(snapshot: &ContextSnapshot) {
    snapshot.restore();
}

/// Clear all context data
pub fn clear_context() {
    CONTEXT_STACK.with(|stack| stack.borrow_mut().clear());
    if let Ok(mut ctx) = GLOBAL_CONTEXT.write() {
        ctx.clear();
    }
}

/// Get the depth of the context stack
pub fn context_depth() -> usize {
    CONTEXT_STACK.with(|stack| stack.borrow().len())
}

/// Create a new context scope that will be automatically popped when dropped
pub struct ContextScope {
    _private: (),
}

impl Default for ContextScope {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextScope {
    pub fn new() -> Self {
        push_context(ContextMap::new());
        Self { _private: () }
    }
}

impl Drop for ContextScope {
    fn drop(&mut self) {
        pop_context();
    }
}

/// Create a new context scope
pub fn create_context_scope() -> ContextScope {
    ContextScope::new()
}
