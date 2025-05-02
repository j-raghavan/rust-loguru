use parking_lot::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::level::LogLevel;
use crate::record::Record;

/// A thread-safe pool of reusable Record objects
pub struct RecordPool {
    /// The pool of available records
    pool: Arc<Mutex<Vec<Record>>>,
    /// The maximum capacity of the pool
    capacity: usize,
    /// The current size of the pool
    size: Arc<AtomicUsize>,
}

impl RecordPool {
    /// Create a new record pool with the specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            pool: Arc::new(Mutex::new(Vec::with_capacity(capacity))),
            capacity,
            size: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Acquire a record from the pool or create a new one
    pub fn acquire(&self) -> PooledRecord {
        let record = {
            let mut guard = self.pool.lock();
            if let Some(record) = guard.pop() {
                self.size.fetch_sub(1, Ordering::Relaxed);
                record
            } else {
                Record::empty()
            }
        };

        PooledRecord {
            record: Some(record),
            pool: self.pool.clone(),
            size: self.size.clone(),
            capacity: self.capacity,
        }
    }

    /// Get the current size of the pool
    pub fn size(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }

    /// Get the capacity of the pool
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Clear the pool, dropping all records
    pub fn clear(&self) {
        let mut guard = self.pool.lock();
        guard.clear();
        self.size.store(0, Ordering::Relaxed);
    }
}

impl Default for RecordPool {
    fn default() -> Self {
        Self::new(128)
    }
}

/// A record acquired from a pool that will be returned when dropped
pub struct PooledRecord {
    /// The record, wrapped in Option for take() in drop
    record: Option<Record>,
    /// Reference to the pool
    pool: Arc<Mutex<Vec<Record>>>,
    /// Reference to the pool size counter
    size: Arc<AtomicUsize>,
    /// Pool capacity
    capacity: usize,
}

impl PooledRecord {
    /// Get a mutable reference to the record
    pub fn get_mut(&mut self) -> &mut Record {
        self.record.as_mut().unwrap()
    }

    /// Get a reference to the record
    pub fn get(&self) -> &Record {
        self.record.as_ref().unwrap()
    }

    /// Take ownership of the record, preventing it from being returned to the pool
    pub fn into_record(mut self) -> Record {
        self.record.take().unwrap()
    }

    /// Set the log level
    pub fn set_level(&mut self, level: LogLevel) {
        if let Some(record) = self.record.as_mut() {
            *record = Record::new(
                level,
                record.message().to_string(),
                Some(record.module().to_string()),
                Some(record.file().to_string()),
                Some(record.line()),
            );
        }
    }

    /// Set the message
    pub fn set_message(&mut self, message: impl Into<String>) {
        if let Some(record) = self.record.as_mut() {
            *record = Record::new(
                record.level(),
                message,
                Some(record.module().to_string()),
                Some(record.file().to_string()),
                Some(record.line()),
            );
        }
    }
}

impl Drop for PooledRecord {
    fn drop(&mut self) {
        // Only return the record to the pool if we still have it
        if let Some(record) = self.record.take() {
            let current_size = self.size.load(Ordering::Relaxed);
            if current_size < self.capacity {
                let mut guard = self.pool.lock();
                guard.push(record);
                self.size.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

impl std::ops::Deref for PooledRecord {
    type Target = Record;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl std::ops::DerefMut for PooledRecord {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_pool_basic() {
        let pool = RecordPool::new(2);
        assert_eq!(pool.size(), 0);

        let mut record = pool.acquire();
        record.set_level(LogLevel::Info);
        record.set_message("test");
        drop(record);

        assert_eq!(pool.size(), 1);
    }

    #[test]
    fn test_record_pool_capacity() {
        let pool = RecordPool::new(1);

        let record1 = pool.acquire();
        let record2 = pool.acquire();

        drop(record1);
        assert_eq!(pool.size(), 1);

        drop(record2);
        assert_eq!(pool.size(), 1); // Still 1, not 2
    }

    #[test]
    fn test_record_pool_clear() {
        let pool = RecordPool::new(2);

        let record1 = pool.acquire();
        let record2 = pool.acquire();

        drop(record1);
        drop(record2);

        assert_eq!(pool.size(), 2);
        pool.clear();
        assert_eq!(pool.size(), 0);
    }

    #[test]
    fn test_pooled_record_into_record() {
        let pool = RecordPool::new(1);

        let record = pool.acquire();
        let _owned = record.into_record();

        assert_eq!(pool.size(), 0); // Record was not returned to pool
    }

    #[test]
    fn test_pooled_record() {
        let pool = RecordPool::new(1);
        let mut record = pool.acquire();

        record.set_level(LogLevel::Info);
        record.set_message("test");

        let owned = record.into_record();
        assert_eq!(owned.level(), LogLevel::Info);
        assert_eq!(owned.message(), "test");
    }
}
