use crate::handler::HandlerRef;
use crate::level::LogLevel;
use std::path::PathBuf;

/// Configuration for the logger.
#[derive(Debug)]
pub struct LoggerConfig {
    /// The minimum log level to record.
    pub level: LogLevel,
    /// The handlers to use for logging.
    pub handlers: Vec<HandlerRef>,
    /// Whether to capture source location information.
    pub capture_source: bool,
    /// Whether to use colors in console output.
    pub use_colors: bool,
    /// The format string for log messages.
    pub format: String,
}

impl Clone for LoggerConfig {
    fn clone(&self) -> Self {
        Self {
            level: self.level,
            handlers: self.handlers.clone(),
            capture_source: self.capture_source,
            use_colors: self.use_colors,
            format: self.format.clone(),
        }
    }
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            handlers: Vec::new(),
            capture_source: true,
            use_colors: true,
            format: "{time} {level} {message}".to_string(),
        }
    }
}

/// Builder for creating logger configurations.
#[derive(Debug)]
pub struct LoggerConfigBuilder {
    config: LoggerConfig,
}

impl Default for LoggerConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LoggerConfigBuilder {
    /// Create a new builder with default configuration.
    pub fn new() -> Self {
        Self {
            config: LoggerConfig::default(),
        }
    }

    /// Set the minimum log level.
    pub fn level(mut self, level: LogLevel) -> Self {
        self.config.level = level;
        self
    }

    /// Add a handler to the configuration.
    pub fn add_handler(mut self, handler: HandlerRef) -> Self {
        self.config.handlers.push(handler);
        self
    }

    /// Set whether to capture source location information.
    pub fn capture_source(mut self, capture: bool) -> Self {
        self.config.capture_source = capture;
        self
    }

    /// Set whether to use colors in console output.
    pub fn use_colors(mut self, use_colors: bool) -> Self {
        self.config.use_colors = use_colors;
        self
    }

    /// Set the format string for log messages.
    pub fn format(mut self, format: String) -> Self {
        self.config.format = format;
        self
    }

    /// Build the final configuration.
    pub fn build(self) -> LoggerConfig {
        self.config
    }
}

/// Helper functions for creating common configurations.
impl LoggerConfig {
    /// Create a basic console configuration.
    pub fn basic_console() -> LoggerConfig {
        LoggerConfigBuilder::new()
            .level(LogLevel::Info)
            .use_colors(true)
            .build()
    }

    /// Create a file logging configuration.
    pub fn file_logging(_path: PathBuf) -> LoggerConfig {
        LoggerConfigBuilder::new()
            .level(LogLevel::Info)
            .use_colors(false)
            .build()
    }

    /// Create a development configuration with detailed logging.
    pub fn development() -> LoggerConfig {
        LoggerConfigBuilder::new()
            .level(LogLevel::Debug)
            .use_colors(true)
            .capture_source(true)
            .format("{time} {level} {file}:{line} {message}".to_string())
            .build()
    }

    /// Create a production configuration with minimal logging.
    pub fn production() -> LoggerConfig {
        LoggerConfigBuilder::new()
            .level(LogLevel::Info)
            .use_colors(false)
            .capture_source(false)
            .format("{time} {level} {message}".to_string())
            .build()
    }
}
