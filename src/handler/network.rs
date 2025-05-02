use std::fmt;
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use crate::formatters::Formatter;
use crate::level::LogLevel;
use crate::record::Record;

use super::{Handler, HandlerError, HandlerFilter};
use std::fmt::Debug;

/// A handler that writes log records to a network socket
pub struct NetworkHandler {
    level: LogLevel,
    enabled: bool,
    formatter: Formatter,
    stream: Arc<Mutex<Option<TcpStream>>>,
    filter: Option<HandlerFilter>,
    batch_buffer: Arc<Mutex<Vec<Record>>>,
    batch_size: Option<usize>,
    addr: String,
}

impl NetworkHandler {
    pub fn new(stream: TcpStream, level: LogLevel) -> Self {
        Self {
            level,
            enabled: true,
            formatter: Formatter::text(),
            stream: Arc::new(Mutex::new(Some(stream))),
            filter: None,
            batch_buffer: Arc::new(Mutex::new(Vec::new())),
            batch_size: None,
            addr: String::new(),
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

    pub fn with_filter(mut self, filter: HandlerFilter) -> Self {
        self.filter = Some(filter);
        self
    }

    pub fn with_batching(mut self, batch_size: usize) -> Self {
        self.batch_size = Some(batch_size);
        self
    }
}

impl Handler for NetworkHandler {
    fn handle(&self, record: &Record) -> Result<(), HandlerError> {
        if !self.enabled || record.level() < self.level {
            return Ok(());
        }
        if let Some(filter) = &self.filter {
            if !(filter)(record) {
                return Ok(());
            }
        }
        if let Some(batch_size) = self.batch_size {
            let mut buffer = self.batch_buffer.lock().unwrap();
            buffer.push(record.clone());
            if buffer.len() >= batch_size {
                let batch = buffer.drain(..).collect::<Vec<_>>();
                drop(buffer);
                return self.handle_batch(&batch);
            }
            return Ok(());
        }
        let formatted = self.formatter.format(record);
        if let Some(ref mut stream) = self.stream.lock().unwrap().as_mut() {
            write!(stream, "{}", formatted).map_err(HandlerError::IoError)?;
            stream.flush().map_err(HandlerError::IoError)?;
            Ok(())
        } else {
            Err(HandlerError::NotInitialized)
        }
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

    fn set_filter(&mut self, filter: Option<HandlerFilter>) {
        self.filter = filter;
    }

    fn filter(&self) -> Option<&HandlerFilter> {
        self.filter.as_ref()
    }

    fn handle_batch(&self, records: &[Record]) -> Result<(), HandlerError> {
        if !self.enabled {
            return Ok(());
        }

        if let Some(ref mut stream) = self.stream.lock().unwrap().as_mut() {
            for record in records {
                if record.level() < self.level {
                    continue;
                }
                if let Some(filter) = &self.filter {
                    if !(filter)(record) {
                        continue;
                    }
                }
                let formatted = self.formatter.format(record);
                write!(stream, "{}", formatted).map_err(HandlerError::IoError)?;
            }
            stream.flush().map_err(HandlerError::IoError)?;
            Ok(())
        } else {
            Err(HandlerError::NotInitialized)
        }
    }

    fn init(&mut self) -> Result<(), HandlerError> {
        let stream = TcpStream::connect(&self.addr).map_err(HandlerError::IoError)?;
        *self.stream.lock().unwrap() = Some(stream);
        Ok(())
    }

    fn flush(&self) -> Result<(), HandlerError> {
        if let Some(ref mut stream) = self.stream.lock().unwrap().as_mut() {
            stream.flush().map_err(HandlerError::IoError)?;
            Ok(())
        } else {
            Err(HandlerError::NotInitialized)
        }
    }

    fn shutdown(&mut self) -> Result<(), HandlerError> {
        self.flush()
    }
}

impl Debug for NetworkHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NetworkHandler")
            .field("level", &self.level)
            .field("enabled", &self.enabled)
            .field("formatter", &self.formatter)
            .field("batch_size", &self.batch_size)
            .finish()
    }
}

impl Clone for NetworkHandler {
    fn clone(&self) -> Self {
        Self {
            level: self.level,
            enabled: self.enabled,
            formatter: self.formatter.clone(),
            stream: self.stream.clone(),
            filter: self.filter.clone(),
            batch_buffer: self.batch_buffer.clone(),
            batch_size: self.batch_size,
            addr: self.addr.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader};
    use std::net::{TcpListener, TcpStream};
    use std::sync::mpsc::channel;
    use std::thread;

    #[test]
    fn test_network_handler_filtering_and_batching() {
        let (tx, rx) = channel();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        // Start server thread that collects all lines until connection closes
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(&mut stream);
            let mut lines = Vec::new();
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) | Err(_) => break, // Connection closed or error
                    Ok(_) => {
                        if !line.is_empty() {
                            lines.push(line);
                        }
                    }
                }
            }
            tx.send(lines).unwrap();
        });

        // Create and configure handler
        let stream = TcpStream::connect(addr).unwrap();
        let filter = std::sync::Arc::new(|record: &Record| record.message().contains("pass"));
        let handler = NetworkHandler::new(stream, LogLevel::Info)
            .with_filter(filter)
            .with_batching(2);

        // Send test records
        let record1 = Record::new(
            LogLevel::Info,
            "should pass",
            None::<String>,
            None::<String>,
            None,
        );
        let record2 = Record::new(
            LogLevel::Info,
            "should fail",
            None::<String>,
            None::<String>,
            None,
        );
        let record3 = Record::new(
            LogLevel::Info,
            "should pass again",
            None::<String>,
            None::<String>,
            None,
        );

        // Handle records and flush
        assert!(handler.handle(&record1).is_ok());
        assert!(handler.handle(&record2).is_ok());
        assert!(handler.handle(&record3).is_ok());
        handler.flush().unwrap();

        // Close the connection to signal server thread
        drop(handler);

        // Verify results
        let lines = rx.recv().unwrap();
        assert!(lines.iter().any(|l| l.contains("should pass")));
        assert!(lines.iter().any(|l| l.contains("should pass again")));
        assert!(!lines.iter().any(|l| l.contains("should fail")));
    }
}
