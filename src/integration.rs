//! Integration module for rust-loguru
//!
//! - Log crate compatibility
//! - Async runtime integrations
//! - Framework middleware

/// Integration with the `log` crate.
pub mod log_compat {
    /// Initialize rust-loguru as the global logger for the `log` crate.
    pub fn init_loguru_as_log() {
        // Use the log_adapter module from the crate root
        crate::log_adapter::init().expect("Failed to set log adapter as global logger");
        let level = crate::logger::global().read().level();
        crate::log_adapter::set_max_level(level);
    }
}

/// Async runtime integrations (e.g., tokio).
pub mod async_runtime {
    /// Integrate with async runtimes (e.g., spawn log flushing tasks).
    #[cfg(feature = "tokio")]
    pub async fn integrate_with_tokio() {
        use crate::logger::global;
        use std::sync::Arc;
        use tokio::time::{sleep, Duration};

        // Example: spawn a background task that could flush logs every 100ms
        tokio::spawn(async move {
            loop {
                // In a real implementation, you might flush async log queues here
                // For now, just sleep to demonstrate integration
                sleep(Duration::from_millis(100)).await;
                // Optionally, call a flush method if implemented
                let _ = global();
            }
        });
    }

    #[cfg(not(feature = "tokio"))]
    pub async fn integrate_with_tokio() {
        panic!("tokio integration requires the 'tokio' feature to be enabled");
    }
}

/// Framework middleware (e.g., for web frameworks).
pub mod middleware {
    /// Middleware for web frameworks (e.g., actix, axum).
    pub fn request_response_logging() {
        // TODO: Implement request/response logging middleware
        unimplemented!("framework middleware not yet implemented");
    }
}
