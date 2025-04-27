use crate::handler::HandlerRef;
use crate::level::LogLevel;
use std::fs;
use std::io;
use std::path::PathBuf;
use toml;

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

    /// Apply environment variable overrides to the configuration.
    ///
    /// Supported environment variables:
    /// - LOGURU_LEVEL (e.g., "Debug", "Info", "Warn", "Error")
    /// - LOGURU_CAPTURE_SOURCE ("true"/"false")
    /// - LOGURU_USE_COLORS ("true"/"false")
    /// - LOGURU_FORMAT (format string)
    pub fn with_env_overrides(mut self) -> Self {
        use std::env;
        if let Ok(level) = env::var("LOGURU_LEVEL") {
            if let Ok(parsed) = level.parse::<LogLevel>() {
                self.config.level = parsed;
            } else {
                // Accept lowercase too
                let level_up = level.to_ascii_uppercase();
                if let Ok(parsed) = level_up.parse::<LogLevel>() {
                    self.config.level = parsed;
                }
            }
        }
        if let Ok(capture_source) = env::var("LOGURU_CAPTURE_SOURCE") {
            self.config.capture_source =
                matches!(capture_source.as_str(), "1" | "true" | "TRUE" | "True");
        }
        if let Ok(use_colors) = env::var("LOGURU_USE_COLORS") {
            self.config.use_colors = matches!(use_colors.as_str(), "1" | "true" | "TRUE" | "True");
        }
        if let Ok(format) = env::var("LOGURU_FORMAT") {
            self.config.format = format;
        }
        self
    }

    /// Build the final configuration.
    pub fn build(self) -> LoggerConfig {
        self.config
    }

    /// Load configuration from a TOML string. Builder/env overrides take precedence.
    ///
    /// Supported TOML keys:
    /// - level (e.g., "Debug", "Info", "Warn", "Error")
    /// - capture_source (bool)
    /// - use_colors (bool)
    /// - format (string)
    pub fn from_toml_str(mut self, toml_str: &str) -> Result<Self, toml::de::Error> {
        let toml_cfg: LoggerConfigToml = toml::from_str(toml_str)?;
        if let Some(level) = toml_cfg.level {
            if let Ok(parsed) = level.parse::<LogLevel>() {
                self.config.level = parsed;
            }
        }
        if let Some(capture_source) = toml_cfg.capture_source {
            self.config.capture_source = capture_source;
        }
        if let Some(use_colors) = toml_cfg.use_colors {
            self.config.use_colors = use_colors;
        }
        if let Some(format) = toml_cfg.format {
            self.config.format = format;
        }
        Ok(self)
    }

    /// Load configuration from a TOML file. Builder/env overrides take precedence.
    pub fn from_toml_file<P: AsRef<std::path::Path>>(self, path: P) -> Result<Self, io::Error> {
        let content = fs::read_to_string(path)?;
        // Propagate TOML parse errors as io::Error
        self.from_toml_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
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

#[derive(serde::Deserialize, Debug, Default)]
struct LoggerConfigToml {
    level: Option<String>,
    capture_source: Option<bool>,
    use_colors: Option<bool>,
    format: Option<String>,
}
