//! Performance benchmarks for rust_loguru
//!
//! This file contains benchmarks for measuring the performance of various
//! aspects of the rust_loguru library, including synchronous and asynchronous
//! logging, different handlers, and concurrency scenarios.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use parking_lot::RwLock;
use rust_loguru::formatters::Formatter;
use rust_loguru::handler::Handler;
use rust_loguru::handler::NullHandler;
use rust_loguru::level::LogLevel;
use rust_loguru::logger::Logger;
use rust_loguru::record::Record;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug)]
struct CountingHandler {
    count: AtomicUsize,
    level: LogLevel,
    enabled: bool,
    formatter: Formatter,
}

impl CountingHandler {
    fn new(level: LogLevel) -> Self {
        Self {
            count: AtomicUsize::new(0),
            level,
            enabled: true,
            formatter: Formatter::text(),
        }
    }

    // fn count(&self) -> usize {
    //     self.count.load(Ordering::Relaxed)
    // }
}

impl Handler for CountingHandler {
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

    fn handle(&self, _record: &Record) -> Result<(), String> {
        self.count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    fn set_filter(&mut self, _filter: Option<Arc<dyn Fn(&Record) -> bool + Send + Sync>>) {
        // No-op for test handler
    }

    fn filter(&self) -> Option<&Arc<dyn Fn(&Record) -> bool + Send + Sync>> {
        None
    }
}

/// Benchmark synchronous logging throughput
fn bench_sync_logging(c: &mut Criterion) {
    let mut group = c.benchmark_group("synchronous_logging");

    // Test different log levels
    for &level in &[
        LogLevel::Debug,
        LogLevel::Info,
        LogLevel::Warning,
        LogLevel::Error,
    ] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", level)),
            &level,
            |b, &level| {
                let mut logger = Logger::new(level);
                let handler = Arc::new(RwLock::new(NullHandler::new(level)));
                logger.add_handler(handler);

                b.iter(|| {
                    let record = Record::new(
                        level,
                        "Benchmark test message",
                        Some("benchmark_module".to_string()),
                        Some("benchmark.rs".to_string()),
                        Some(42),
                    );
                    black_box(logger.log(&record))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark asynchronous logging throughput
fn bench_async_logging(c: &mut Criterion) {
    let mut group = c.benchmark_group("asynchronous_logging");

    // Test different queue sizes
    for &queue_size in &[100, 1000, 10000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("queue_{}", queue_size)),
            &queue_size,
            |b, &queue_size| {
                let mut logger = Logger::new(LogLevel::Info);
                let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
                logger.add_handler(handler);
                logger.set_async(true, Some(queue_size));

                b.iter(|| {
                    let record = Record::new(
                        LogLevel::Info,
                        "Benchmark test message",
                        Some("benchmark_module".to_string()),
                        Some("benchmark.rs".to_string()),
                        Some(42),
                    );
                    black_box(logger.log(&record))
                });

                // Disable async logging after benchmark
                logger.set_async(false, None);
            },
        );
    }

    // Test different worker thread counts
    for &workers in &[1, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("workers_{}", workers)),
            &workers,
            |b, &workers| {
                let mut logger = Logger::new(LogLevel::Info);
                let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
                logger.add_handler(handler);
                logger.set_async(true, Some(10000));
                logger.set_worker_threads(workers);

                b.iter(|| {
                    let record = Record::new(
                        LogLevel::Info,
                        "Benchmark test message",
                        Some("benchmark_module".to_string()),
                        Some("benchmark.rs".to_string()),
                        Some(42),
                    );
                    black_box(logger.log(&record))
                });

                // Disable async logging after benchmark
                logger.set_async(false, None);
            },
        );
    }

    group.finish();
}

/// Benchmark logger with different handlers
fn bench_handler_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("handler_types");

    // Benchmark with NullHandler
    group.bench_function("null_handler", |b| {
        let mut logger = Logger::new(LogLevel::Info);
        let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
        logger.add_handler(handler);

        b.iter(|| {
            let record = Record::new(
                LogLevel::Info,
                "Benchmark test message",
                Some("benchmark_module".to_string()),
                Some("benchmark.rs".to_string()),
                Some(42),
            );
            black_box(logger.log(&record))
        });
    });

    // Benchmark with CountingHandler
    group.bench_function("counting_handler", |b| {
        let mut logger = Logger::new(LogLevel::Info);
        let handler = Arc::new(RwLock::new(CountingHandler::new(LogLevel::Info)));
        logger.add_handler(handler);

        b.iter(|| {
            let record = Record::new(
                LogLevel::Info,
                "Benchmark test message",
                Some("benchmark_module".to_string()),
                Some("benchmark.rs".to_string()),
                Some(42),
            );
            black_box(logger.log(&record))
        });
    });

    // Benchmark with multiple handlers
    group.bench_function("multiple_handlers", |b| {
        let mut logger = Logger::new(LogLevel::Info);
        let handler1 = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
        let handler2 = Arc::new(RwLock::new(CountingHandler::new(LogLevel::Info)));
        logger.add_handler(handler1);
        logger.add_handler(handler2);

        b.iter(|| {
            let record = Record::new(
                LogLevel::Info,
                "Benchmark test message",
                Some("benchmark_module".to_string()),
                Some("benchmark.rs".to_string()),
                Some(42),
            );
            black_box(logger.log(&record))
        });
    });

    group.finish();
}

/// Benchmark different message sizes
fn bench_message_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_sizes");

    // Generate messages of different sizes
    let small_message = "Small message".to_string();
    let medium_message =
        "Medium sized message with some additional text to make it longer".to_string();
    let large_message = "A".repeat(1000);

    for (name, message) in [
        ("small", small_message),
        ("medium", medium_message),
        ("large", large_message),
    ] {
        group.bench_with_input(BenchmarkId::from_parameter(name), &message, |b, message| {
            let mut logger = Logger::new(LogLevel::Info);
            let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
            logger.add_handler(handler);

            b.iter(|| {
                let record = Record::new(
                    LogLevel::Info,
                    message.clone(),
                    Some("benchmark_module".to_string()),
                    Some("benchmark.rs".to_string()),
                    Some(42),
                );
                black_box(logger.log(&record))
            });
        });

        // Also benchmark with async logger
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_async", name)),
            &message,
            |b, message| {
                let mut logger = Logger::new(LogLevel::Info);
                let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
                logger.add_handler(handler);
                logger.set_async(true, Some(10000));

                b.iter(|| {
                    let record = Record::new(
                        LogLevel::Info,
                        message.clone(),
                        Some("benchmark_module".to_string()),
                        Some("benchmark.rs".to_string()),
                        Some(42),
                    );
                    black_box(logger.log(&record))
                });

                // Disable async logging after benchmark
                logger.set_async(false, None);
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent logging from multiple threads
fn bench_concurrent_logging(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_logging");

    for &thread_count in &[2, 4, 8, 16] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("threads_{}", thread_count)),
            &thread_count,
            |b, &thread_count| {
                b.iter(|| {
                    // Create a shared logger
                    let mut logger = Logger::new(LogLevel::Info);
                    let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
                    logger.add_handler(handler);
                    let logger = Arc::new(RwLock::new(logger));

                    // Create threads that log concurrently
                    let log_count = 1000 / thread_count; // Divide work among threads
                    let threads: Vec<_> = (0..thread_count)
                        .map(|thread_id| {
                            let logger = Arc::clone(&logger);
                            thread::spawn(move || {
                                for i in 0..log_count {
                                    let record = Record::new(
                                        LogLevel::Info,
                                        format!("Thread {} message {}", thread_id, i),
                                        Some("benchmark_module".to_string()),
                                        Some("benchmark.rs".to_string()),
                                        Some(42),
                                    );
                                    logger.read().log(&record);
                                }
                            })
                        })
                        .collect();

                    // Wait for all threads to finish
                    for thread in threads {
                        thread.join().unwrap();
                    }
                });
            },
        );

        // Also benchmark with async logger
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("threads_{}_async", thread_count)),
            &thread_count,
            |b, &thread_count| {
                b.iter(|| {
                    // Create a shared logger with async enabled
                    let mut logger = Logger::new(LogLevel::Info);
                    let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
                    logger.add_handler(handler);
                    logger.set_async(true, Some(10000));
                    let logger = Arc::new(RwLock::new(logger));

                    // Create threads that log concurrently
                    let log_count = 1000 / thread_count; // Divide work among threads
                    let threads: Vec<_> = (0..thread_count)
                        .map(|thread_id| {
                            let logger = Arc::clone(&logger);
                            thread::spawn(move || {
                                for i in 0..log_count {
                                    let record = Record::new(
                                        LogLevel::Info,
                                        format!("Thread {} message {}", thread_id, i),
                                        Some("benchmark_module".to_string()),
                                        Some("benchmark.rs".to_string()),
                                        Some(42),
                                    );
                                    logger.read().log(&record);
                                }
                            })
                        })
                        .collect();

                    // Wait for all threads to finish
                    for thread in threads {
                        thread.join().unwrap();
                    }

                    // Give async threads a moment to process
                    thread::sleep(Duration::from_millis(50));

                    // Disable async logging after benchmark
                    logger.write().set_async(false, None);
                });
            },
        );
    }

    group.finish();
}

/// Measure throughput for high-volume logging scenarios
#[test]
fn test_high_volume_throughput() {
    // Create a logger that discards logs
    let mut logger = Logger::new(LogLevel::Info);
    let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
    logger.add_handler(handler);

    // Test sync logging throughput
    let iterations = 100000;
    let start = Instant::now();

    for i in 0..iterations {
        let record = Record::new(
            LogLevel::Info,
            format!("High volume test message {}", i),
            Some("test_module".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );
        logger.log(&record);
    }

    let sync_duration = start.elapsed();
    let sync_throughput = iterations as f64 / sync_duration.as_secs_f64();

    println!("Sync logging throughput: {:.2} logs/sec", sync_throughput);

    // Test async logging throughput
    logger.set_async(true, Some(iterations));
    let start = Instant::now();

    for i in 0..iterations {
        let record = Record::new(
            LogLevel::Info,
            format!("High volume test message {}", i),
            Some("test_module".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );
        logger.log(&record);
    }

    let async_duration = start.elapsed();
    let async_throughput = iterations as f64 / async_duration.as_secs_f64();

    // Wait for async logs to process
    thread::sleep(Duration::from_millis(200));

    println!("Async logging throughput: {:.2} logs/sec", async_throughput);
    println!(
        "Performance improvement: {:.2}x",
        async_throughput / sync_throughput
    );

    // Disable async logging
    logger.set_async(false, None);
}

/// Test different worker thread counts
#[test]
fn test_worker_thread_scaling() {
    let iterations = 50000;
    let message = "Worker thread scaling test message";

    println!("Testing worker thread scaling:");
    println!("-----------------------------");

    for workers in [1, 2, 4, 8, 16] {
        // Create a logger
        let mut logger = Logger::new(LogLevel::Info);
        let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
        logger.add_handler(handler);

        // Configure async logging with specified worker count
        logger.set_async(true, Some(iterations));
        logger.set_worker_threads(workers);

        // Measure throughput
        let start = Instant::now();

        for _ in 0..iterations {
            let record = Record::new(
                LogLevel::Info,
                message,
                Some("test_module".to_string()),
                Some("test.rs".to_string()),
                Some(42),
            );
            logger.log(&record);
        }

        let duration = start.elapsed();
        let throughput = iterations as f64 / duration.as_secs_f64();

        // Wait for async logs to process
        thread::sleep(Duration::from_millis(200));

        println!("{} workers: {:.2} logs/sec", workers, throughput);

        // Disable async logging
        logger.set_async(false, None);
    }
}

/// Test compile-time filtering impact
#[test]
fn test_compile_time_filtering() {
    // This is more of a demonstration than a test
    // The real benefits would be seen with feature flags

    let mut logger = Logger::new(LogLevel::Info);
    let handler = Arc::new(RwLock::new(NullHandler::new(LogLevel::Info)));
    logger.add_handler(handler);

    let iterations = 100000;

    // Test with runtime checks only
    let start = Instant::now();

    for i in 0..iterations {
        let record = Record::new(
            LogLevel::Debug, // Below Info, should be filtered
            format!("Runtime filtering test {}", i),
            Some("test_module".to_string()),
            Some("test.rs".to_string()),
            Some(42),
        );
        logger.log(&record);
    }

    let runtime_duration = start.elapsed();

    // Test with compile-time checks (simulated)
    // In a real scenario, these would be optimized away at compile time
    let start = Instant::now();

    for i in 0..iterations {
        if rust_loguru::compile_time_level_enabled!(LogLevel::Debug) {
            // This would be optimized away at compile time with feature flags
            let record = Record::new(
                LogLevel::Debug,
                format!("Compile-time filtering test {}", i),
                Some("test_module".to_string()),
                Some("test.rs".to_string()),
                Some(42),
            );
            logger.log(&record);
        }
    }

    let compile_time_duration = start.elapsed();

    println!(
        "Runtime filtering: {:.2} µs",
        runtime_duration.as_micros() as f64 / iterations as f64
    );
    println!(
        "Compile-time filtering: {:.2} µs",
        compile_time_duration.as_micros() as f64 / iterations as f64
    );
    println!(
        "Performance difference: {:.2}x",
        runtime_duration.as_secs_f64() / compile_time_duration.as_secs_f64()
    );
}

criterion_group!(
    benches,
    bench_sync_logging,
    bench_async_logging,
    bench_handler_types,
    bench_message_sizes,
    bench_concurrent_logging,
);
criterion_main!(benches);
