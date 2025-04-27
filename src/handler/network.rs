use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use crate::formatters::Formatter;
use crate::level::LogLevel;
use crate::record::Record;

use super::Handler;
use std::fmt::Debug;

/// A handler that writes log records to a network socket
#[derive(Debug, Clone)]
pub struct NetworkHandler {
    level: LogLevel,
    enabled: bool,
    formatter: Formatter,
    stream: Arc<Mutex<TcpStream>>,
}

impl NetworkHandler {
    pub fn new(stream: TcpStream, level: LogLevel) -> Self {
        Self {
            level,
            enabled: true,
            formatter: Formatter::text(),
            stream: Arc::new(Mutex::new(stream)),
        }
    }

    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }

    pub fn with_formatter(mut self, formatter: Formatter) -> Self {
        self.formatter = formatter;
        self
    }

    pub fn with_colors(mut self, use_colors: bool) -> Self {
        self.formatter = self.formatter.with_colors(use_colors);
        self
    }

    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.formatter = self.formatter.with_pattern(pattern);
        self
    }

    pub fn with_format<F>(mut self, format_fn: F) -> Self
    where
        F: Fn(&Record) -> String + Send + Sync + 'static,
    {
        self.formatter = self.formatter.with_format(format_fn);
        self
    }
}

impl Handler for NetworkHandler {
    fn handle(&self, record: &Record) -> Result<(), String> {
        if !self.enabled || record.level() < self.level {
            return Ok(());
        }

        let formatted = self.formatter.format(record);
        let mut stream = self
            .stream
            .lock()
            .map_err(|e| format!("Failed to lock stream: {}", e))?;
        writeln!(stream, "{}", formatted)
            .map_err(|e| format!("Failed to write to network: {}", e))?;
        Ok(())
    }

    fn level(&self) -> LogLevel {
        self.level
    }

    fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn formatter(&self) -> &Formatter {
        &self.formatter
    }

    fn set_formatter(&mut self, formatter: Formatter) {
        self.formatter = formatter;
    }
}
