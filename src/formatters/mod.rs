pub mod json;
pub mod template;
pub mod text;
pub mod util;

use crate::record::Record;
use std::fmt::Debug;
use std::sync::Arc;

/// A type alias for a format function
pub type FormatFn = Arc<dyn Fn(&Record) -> String + Send + Sync>;

/// A trait for formatters that format log records
pub trait FormatterTrait: Send + Sync + Debug {
    /// Format a single record into a string
    fn fmt(&self, record: &Record) -> String;

    /// Format multiple records into a string (default implementation)
    fn fmt_batch(&self, records: &[Record]) -> String {
        records
            .iter()
            .map(|record| FormatterTrait::fmt(self, record))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Enable or disable colored output
    fn with_colors(&mut self, use_colors: bool);

    /// Enable or disable timestamp output
    fn with_timestamp(&mut self, include_timestamp: bool);

    /// Enable or disable level output
    fn with_level(&mut self, include_level: bool);

    /// Enable or disable module output
    fn with_module(&mut self, include_module: bool);

    /// Enable or disable location output
    fn with_location(&mut self, include_location: bool);

    /// Set the pattern for formatting
    fn with_pattern(&mut self, pattern: String);

    /// Set a custom format function
    fn with_format(&mut self, format_fn: FormatFn);

    /// Clone the formatter into a boxed trait object
    fn box_clone(&self) -> Box<dyn FormatterTrait + Send + Sync>;
}

/// A formatter that can format log records
#[derive(Debug)]
pub struct Formatter {
    inner: Box<dyn FormatterTrait + Send + Sync>,
}

impl Clone for Formatter {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.box_clone(),
        }
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Self::text()
    }
}

impl Formatter {
    /// Create a new text formatter
    pub fn text() -> Self {
        Self {
            inner: Box::new(crate::formatters::text::TextFormatter::default()),
        }
    }

    /// Create a new JSON formatter
    pub fn json() -> Self {
        Self {
            inner: Box::new(crate::formatters::json::JsonFormatter::default()),
        }
    }

    /// Create a new template formatter
    pub fn template(template: impl Into<String>) -> Self {
        Self {
            inner: Box::new(crate::formatters::template::TemplateFormatter::new(
                template,
            )),
        }
    }

    /// Format a single record into a string
    pub fn format(&self, record: &Record) -> String {
        FormatterTrait::fmt(&*self.inner, record)
    }

    /// Format multiple records into a string
    pub fn format_batch(&self, records: &[Record]) -> String {
        self.inner.fmt_batch(records)
    }

    /// Enable or disable colored output
    pub fn with_colors(mut self, use_colors: bool) -> Self {
        self.inner.with_colors(use_colors);
        self
    }

    /// Enable or disable timestamp output
    pub fn with_timestamp(mut self, include_timestamp: bool) -> Self {
        self.inner.with_timestamp(include_timestamp);
        self
    }

    /// Enable or disable level output
    pub fn with_level(mut self, include_level: bool) -> Self {
        self.inner.with_level(include_level);
        self
    }

    /// Enable or disable module output
    pub fn with_module(mut self, include_module: bool) -> Self {
        self.inner.with_module(include_module);
        self
    }

    /// Enable or disable location output
    pub fn with_location(mut self, include_location: bool) -> Self {
        self.inner.with_location(include_location);
        self
    }

    /// Set the pattern for formatting
    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.inner.with_pattern(pattern.into());
        self
    }

    /// Set a custom format function
    pub fn with_format<F>(mut self, format_fn: F) -> Self
    where
        F: Fn(&Record) -> String + Send + Sync + 'static,
    {
        self.inner.with_format(Arc::new(format_fn));
        self
    }
}

// Re-export formatters
pub use self::json::JsonFormatter;
pub use self::template::TemplateFormatter;
pub use self::text::TextFormatter;
