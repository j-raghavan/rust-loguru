use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam_channel::{bounded, Receiver, Sender};
use parking_lot::RwLock;

use crate::level::LogLevel;
use crate::record::Record;
use crate::handler::{Handler, HandlerRef, HandlerError};
use crate::formatters::Formatter;
use crate::handler::HandlerFilter;

const DEFAULT_BATCH_SIZE: usize = 32;
const DEFAULT_FLUSH_INTERVAL: Duration = Duration::from_millis(100);

/// Async logging command type
#[derive(Debug)]
pub enum AsyncCommand {
    /// Log a record
    Log(Record),
    /// Flush all handlers
    Flush,
    /// Shut down the async logger
    Shutdown,
}

/// Handle to the async logger
#[derive(Clone, Debug)]
pub struct AsyncLoggerHandle {
    /// Channel for sending commands to the async logger
    sender: Sender<AsyncCommand>,
    /// Flag indicating whether the async logger is running
    running: Arc<AtomicBool>,
    /// Number of queued records
    queued_records: Arc<AtomicUsize>,
}

impl AsyncLoggerHandle {
    /// Log a record
    pub fn log(&self, record: Record) -> bool {
        if !self.running.load(Ordering::Relaxed) {
            return false;
        }

        match self.sender.try_send(AsyncCommand::Log(record)) {
            Ok(_) => {
                self.queued_records.fetch_add(1, Ordering::Relaxed);
                true
            }
            Err(_) => false,
        }
    }

    /// Flush all handlers
    pub fn flush(&self) -> bool {
        if !self.running.load(Ordering::Relaxed) {
            return false;
        }

        self.sender.try_send(AsyncCommand::Flush).is_ok()
    }

    /// Shut down the async logger
    pub fn shutdown(&self) {
        if !self.running.load(Ordering::Relaxed) {
            return;
        }

        // Send shutdown command and update running flag
        let _ = self.sender.send(AsyncCommand::Shutdown);
        self.running.store(false, Ordering::Relaxed);
    }

    /// Get the number of queued records
    pub fn queued_records(&self) -> usize {
        self.queued_records.load(Ordering::Relaxed)
    }
}

impl Drop for AsyncLoggerHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Builder for the async logger
pub struct AsyncLoggerBuilder {
    /// Queue size
    queue_size: usize,
    /// Handler registry
    handlers: Vec<HandlerRef>,
    /// Log level
    level: LogLevel,
    /// Number of worker threads
    workers: usize,
    /// Batch size for processing records
    batch_size: usize,
    /// Flush interval
    flush_interval: Duration,
}

impl AsyncLoggerBuilder {
    /// Create a new async logger builder
    pub fn new() -> Self {
        Self {
            queue_size: 10000,
            handlers: Vec::new(),
            level: LogLevel::Info,
            workers: 1,
            batch_size: DEFAULT_BATCH_SIZE,
            flush_interval: DEFAULT_FLUSH_INTERVAL,
        }
    }

    /// Set the queue size
    pub fn with_queue_size(mut self, queue_size: usize) -> Self {
        self.queue_size = queue_size;
        self
    }

    /// Set the handlers
    pub fn with_handlers(mut self, handlers: Vec<HandlerRef>) -> Self {
        self.handlers = handlers;
        self
    }

    /// Set the log level
    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }

    /// Set the number of worker threads
    pub fn with_workers(mut self, workers: usize) -> Self {
        self.workers = workers;
        self
    }

    /// Set the batch size
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }

    /// Set the flush interval
    pub fn with_flush_interval(mut self, flush_interval: Duration) -> Self {
        self.flush_interval = flush_interval;
        self
    }

    /// Build the async logger
    pub fn build(self) -> AsyncLoggerHandle {
        // Create a channel for sending commands to the worker thread
        let (sender, receiver) = bounded(self.queue_size);

        // Create a running flag and queued records counter
        let running = Arc::new(AtomicBool::new(true));
        let queued_records = Arc::new(AtomicUsize::new(0));

        // Spawn the worker threads
        let mut workers = Vec::with_capacity(self.workers);
        for _ in 0..self.workers {
            let worker = AsyncWorker::new(
                receiver.clone(),
                self.handlers.clone(),
                self.level,
                running.clone(),
                queued_records.clone(),
                self.batch_size,
                self.flush_interval,
            );
            workers.push(worker.spawn());
        }

        // Create the async logger handle
        AsyncLoggerHandle {
            sender,
            running,
            queued_records,
        }
    }
}

/// Worker thread for processing async log records
struct AsyncWorker {
    /// Channel for receiving commands
    receiver: Receiver<AsyncCommand>,
    /// Handler registry
    handlers: Vec<HandlerRef>,
    /// Log level
    level: LogLevel,
    /// Running flag
    running: Arc<AtomicBool>,
    /// Queued records counter
    queued_records: Arc<AtomicUsize>,
    /// Batch size
    batch_size: usize,
    /// Flush interval
    flush_interval: Duration,
}

impl AsyncWorker {
    /// Create a new worker
    fn new(
        receiver: Receiver<AsyncCommand>,
        handlers: Vec<HandlerRef>,
        level: LogLevel,
        running: Arc<AtomicBool>,
        queued_records: Arc<AtomicUsize>,
        batch_size: usize,
        flush_interval: Duration,
    ) -> Self {
        Self {
            receiver,
            handlers,
            level,
            running,
            queued_records,
            batch_size,
            flush_interval,
        }
    }

    /// Spawn the worker thread
    fn spawn(self) -> JoinHandle<()> {
        thread::spawn(move || self.run())
    }

    /// Run the worker loop
    fn run(mut self) {
        // Preallocate batch buffer
        let mut batch = Vec::with_capacity(self.batch_size);
        let mut last_flush = std::time::Instant::now();

        while self.running.load(Ordering::Relaxed) {
            match self.receiver.recv_timeout(self.flush_interval) {
                Ok(AsyncCommand::Log(record)) => {
                    batch.push(record);
                    self.queued_records.fetch_sub(1, Ordering::Relaxed);

                    // Process batch if full
                    if batch.len() >= self.batch_size {
                        self.process_batch(&mut batch);
                        last_flush = std::time::Instant::now();
                    }
                }
                Ok(AsyncCommand::Flush) => {
                    self.process_batch(&mut batch);
                    self.flush_handlers();
                    last_flush = std::time::Instant::now();
                }
                Ok(AsyncCommand::Shutdown) => {
                    self.process_batch(&mut batch);
                    self.flush_handlers();
                    break;
                }
                Err(_) => {
                    // Check if we need to flush based on time
                    if !batch.is_empty() && last_flush.elapsed() >= self.flush_interval {
                        self.process_batch(&mut batch);
                        last_flush = std::time::Instant::now();
                    }
                }
            }
        }
    }

    /// Process a batch of records
    fn process_batch(&self, batch: &mut Vec<Record>) {
        if batch.is_empty() {
            return;
        }

        // Process records through handlers
        for handler in &self.handlers {
            let guard = handler.read();
            if guard.is_enabled() && self.level >= guard.level() {
                let _ = guard.handle_batch(batch);
            }
        }

        batch.clear();
    }

    /// Flush all handlers
    fn flush_handlers(&self) {
        for handler in &self.handlers {
            let guard = handler.read();
            if guard.is_enabled() {
                let _ = guard.flush();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use crate::handler::Handler;
    use crate::formatters::Formatter;
    use crate::handler::HandlerFilter;
    
    #[derive(Debug)]
    struct TestHandler {
        records: RwLock<Vec<Record>>,
        enabled: bool,
        level: LogLevel,
        formatter: Formatter,
    }
    
    impl Handler for TestHandler {
        fn handle(&self, record: &Record) -> Result<(), HandlerError> {
            self.records.write().push(record.clone());
            Ok(())
        }
        
        fn handle_batch(&self, records: &[Record]) -> Result<(), HandlerError> {
            let mut guard = self.records.write();
            guard.extend_from_slice(records);
            Ok(())
        }
        
        fn is_enabled(&self) -> bool {
            self.enabled
        }

        fn level(&self) -> LogLevel {
            self.level
        }

        fn set_level(&mut self, level: LogLevel) {
            self.level = level;
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

        fn set_filter(&mut self, _filter: Option<HandlerFilter>) {
            // Not used in test
        }

        fn filter(&self) -> Option<&HandlerFilter> {
            None
        }
    }
    
    #[test]
    fn test_async_logger_basic() {
        let handler = Arc::new(TestHandler {
            records: RwLock::new(Vec::new()),
            enabled: true,
            level: LogLevel::Info,
            formatter: Formatter::default(),
        });
        
        let logger = AsyncLogger::new(
            LogLevel::Info,
            vec![Arc::new(RwLock::new(handler.clone()))],
            100,
            2
        );
        
        // Send some records
        for i in 0..10 {
            let record = Record::new(
                LogLevel::Info,
                format!("test message {}", i),
                None,
                None,
                None,
            );
            assert!(logger.log(record));
        }
        
        // Give time for processing
        thread::sleep(Duration::from_millis(200));
        
        // Verify records were processed
        let records = handler.records.read();
        assert_eq!(records.len(), 10);
    }
    
    #[test]
    fn test_async_logger_level_filtering() {
        let handler = Arc::new(TestHandler {
            records: RwLock::new(Vec::new()),
            enabled: true,
            level: LogLevel::Info,
            formatter: Formatter::default(),
        });
        
        let logger = AsyncLogger::new(
            LogLevel::Info,
            vec![Arc::new(RwLock::new(handler.clone()))],
            100,
            2
        );
        
        // Send records with different levels
        let debug_record = Record::new(
            LogLevel::Debug,
            "debug message",
            None,
            None,
            None,
        );
        let info_record = Record::new(
            LogLevel::Info,
            "info message",
            None,
            None,
            None,
        );
        
        assert!(!logger.log(debug_record)); // Should be filtered
        assert!(logger.log(info_record));   // Should pass
        
        thread::sleep(Duration::from_millis(200));
        
        let records = handler.records.read();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].level(), LogLevel::Info);
    }
    
    #[test]
    fn test_async_logger_shutdown() {
        let handler = Arc::new(TestHandler {
            records: RwLock::new(Vec::new()),
            enabled: true,
            level: LogLevel::Info,
            formatter: Formatter::default(),
        });
        
        let logger = AsyncLogger::new(
            LogLevel::Info,
            vec![Arc::new(RwLock::new(handler.clone()))],
            100,
            2
        );
        
        // Send some records
        for i in 0..5 {
            let record = Record::new(
                LogLevel::Info,
                format!("test message {}", i),
                None,
                None,
                None,
            );
            logger.log(record);
        }
        
        // Shutdown logger
        logger.shutdown();
        
        // Give time for final processing
        thread::sleep(Duration::from_millis(200));
        
        let records = handler.records.read();
        assert_eq!(records.len(), 5);
    }
} 