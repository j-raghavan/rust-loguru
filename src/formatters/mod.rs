use crate::record::Record;
use std::fmt::Debug;

/// A formatter that can format log records
#[derive(Debug, Clone)]
pub enum Formatter {
    Text(TextFormatter),
    Json(JsonFormatter),
    Template(TemplateFormatter),
}

impl Formatter {
    pub fn text() -> Self {
        Self::Text(TextFormatter::new())
    }

    pub fn json() -> Self {
        Self::Json(JsonFormatter::new())
    }

    pub fn template() -> Self {
        Self::Template(TemplateFormatter::new())
    }

    pub fn format(&self, record: &Record) -> String {
        match self {
            Self::Text(f) => f.format(record),
            Self::Json(f) => f.format(record),
            Self::Template(f) => f.format(record),
        }
    }

    pub fn with_colors(self, use_colors: bool) -> Self {
        match self {
            Self::Text(f) => Self::Text(f.with_colors(use_colors)),
            Self::Json(f) => Self::Json(f.with_colors(use_colors)),
            Self::Template(f) => Self::Template(f.with_colors(use_colors)),
        }
    }

    pub fn with_timestamp(self, include_timestamp: bool) -> Self {
        match self {
            Self::Text(f) => Self::Text(f.with_timestamp(include_timestamp)),
            Self::Json(f) => Self::Json(f.with_timestamp(include_timestamp)),
            Self::Template(f) => Self::Template(f.with_timestamp(include_timestamp)),
        }
    }

    pub fn with_level(self, include_level: bool) -> Self {
        match self {
            Self::Text(f) => Self::Text(f.with_level(include_level)),
            Self::Json(f) => Self::Json(f.with_level(include_level)),
            Self::Template(f) => Self::Template(f.with_level(include_level)),
        }
    }

    pub fn with_module(self, include_module: bool) -> Self {
        match self {
            Self::Text(f) => Self::Text(f.with_module(include_module)),
            Self::Json(f) => Self::Json(f.with_module(include_module)),
            Self::Template(f) => Self::Template(f.with_module(include_module)),
        }
    }

    pub fn with_location(self, include_location: bool) -> Self {
        match self {
            Self::Text(f) => Self::Text(f.with_location(include_location)),
            Self::Json(f) => Self::Json(f.with_location(include_location)),
            Self::Template(f) => Self::Template(f.with_location(include_location)),
        }
    }

    pub fn with_pattern(self, pattern: impl Into<String>) -> Self {
        match self {
            Self::Text(f) => Self::Text(f.with_pattern(pattern)),
            Self::Json(f) => Self::Json(f.with_pattern(pattern)),
            Self::Template(f) => Self::Template(f.with_pattern(pattern)),
        }
    }

    pub fn with_format<F>(self, format_fn: F) -> Self
    where
        F: Fn(&Record) -> String + Send + Sync + 'static,
    {
        match self {
            Self::Text(f) => Self::Text(f.with_format(format_fn)),
            Self::Json(f) => Self::Json(f.with_format(format_fn)),
            Self::Template(f) => Self::Template(f.with_format(format_fn)),
        }
    }
}

/// A trait for formatters that format log records
pub trait FormatterTrait: Send + Sync + Debug {
    /// Format a log record
    fn format(&self, record: &Record) -> String;

    /// Set whether to use colors
    fn with_colors(self, use_colors: bool) -> Self
    where
        Self: Sized;

    /// Set whether to include timestamps
    fn with_timestamp(self, include_timestamp: bool) -> Self
    where
        Self: Sized;

    /// Set whether to include the log level
    fn with_level(self, include_level: bool) -> Self
    where
        Self: Sized;

    /// Set whether to include the module path
    fn with_module(self, include_module: bool) -> Self
    where
        Self: Sized;

    /// Set whether to include the file and line number
    fn with_location(self, include_location: bool) -> Self
    where
        Self: Sized;

    /// Set the format pattern
    fn with_pattern(self, pattern: impl Into<String>) -> Self
    where
        Self: Sized;

    /// Set a custom format function
    fn with_format<F>(self, format_fn: F) -> Self
    where
        F: Fn(&Record) -> String + Send + Sync + 'static,
        Self: Sized;

    /// Clone the formatter
    fn box_clone(&self) -> Box<dyn FormatterTrait + Send + Sync>;
}

pub mod json;
pub mod template;
pub mod text;
pub mod util;

pub use self::json::JsonFormatter;
pub use self::template::TemplateFormatter;
pub use self::text::TextFormatter;
