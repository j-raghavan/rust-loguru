use criterion::{
    criterion_group, criterion_main, BenchmarkId, Criterion, SamplingMode, Throughput,
};
use std::io::sink;
use std::sync::{Arc, Barrier, Once};
use std::thread;
use std::time::Duration;

// Import libraries to benchmark
use log::{Level as LogLevel, LevelFilter};
use rust_loguru::handler::{new_handler_ref, NullHandler};
use rust_loguru::{LogLevel as LoguruLogLevel, Logger};
use slog::{Drain, Level as SlogLevel, Logger as SlogLogger};
use tracing::Level as TracingLevel;

// Ensure loggers are only initialized once
static LOG_INIT: Once = Once::new();
static TRACING_INIT: Once = Once::new();

// Message generator for creating test messages
fn generate_message(size: usize) -> String {
    let base = "This is a concurrent logging test message. ";

    if size <= base.len() {
        return base[0..size].to_string();
    }

    let mut message = String::with_capacity(size);
    while message.len() < size {
        message.push_str(base);
    }
    message.truncate(size);
    message
}

// Setup loggers
fn setup_log(level: LogLevel) {
    LOG_INIT.call_once(|| {
        let _ = env_logger::Builder::new()
            .filter_level(LevelFilter::Trace)
            .filter_module(
                "concurrent_benchmark",
                match level {
                    LogLevel::Error => LevelFilter::Error,
                    LogLevel::Warn => LevelFilter::Warn,
                    LogLevel::Info => LevelFilter::Info,
                    LogLevel::Debug => LevelFilter::Debug,
                    LogLevel::Trace => LevelFilter::Trace,
                },
            )
            .target(env_logger::Target::Pipe(Box::new(sink())))
            .is_test(true)
            .try_init();
    });
}

fn setup_slog(level: SlogLevel) -> SlogLogger {
    let drain = slog::Discard.fuse();
    let drain = drain.filter_level(level).fuse();
    SlogLogger::root(drain, slog::o!())
}

fn setup_tracing(level: TracingLevel) {
    TRACING_INIT.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_max_level(level)
            .with_writer(sink)
            .with_ansi(false)
            .try_init();
    });
}

fn setup_loguru(level: LoguruLogLevel) -> Logger {
    let mut logger = Logger::new(level.into());
    logger.add_handler(new_handler_ref(NullHandler::new(level.into())));
    logger
}

// Concurrent logging functions
fn bench_log_concurrent(
    level: LogLevel,
    message: &str,
    thread_count: usize,
    logs_per_thread: usize,
) {
    let barrier = Arc::new(Barrier::new(thread_count + 1));
    let mut handles = Vec::with_capacity(thread_count);

    for thread_id in 0..thread_count {
        let thread_barrier = Arc::clone(&barrier);
        let thread_message = format!("Thread {} - {}", thread_id, message);

        let handle = thread::spawn(move || {
            // Wait for all threads to be ready
            thread_barrier.wait();

            // Log messages at the specified level
            for i in 0..logs_per_thread {
                let log_message = format!("{} - Log {}", thread_message, i);
                match level {
                    LogLevel::Error => log::error!("{}", log_message),
                    LogLevel::Warn => log::warn!("{}", log_message),
                    LogLevel::Info => log::info!("{}", log_message),
                    LogLevel::Debug => log::debug!("{}", log_message),
                    LogLevel::Trace => log::trace!("{}", log_message),
                }
            }
        });

        handles.push(handle);
    }

    // Start all threads simultaneously
    barrier.wait();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
}

fn bench_slog_concurrent(
    logger: &SlogLogger,
    level: SlogLevel,
    message: &str,
    thread_count: usize,
    logs_per_thread: usize,
) {
    let logger = Arc::new(logger.clone());
    let barrier = Arc::new(Barrier::new(thread_count + 1));
    let mut handles = Vec::with_capacity(thread_count);

    for thread_id in 0..thread_count {
        let thread_logger = Arc::clone(&logger);
        let thread_barrier = Arc::clone(&barrier);
        let thread_message = format!("Thread {} - {}", thread_id, message);

        let handle = thread::spawn(move || {
            // Wait for all threads to be ready
            thread_barrier.wait();

            // Log messages at the specified level
            for i in 0..logs_per_thread {
                let log_message = format!("{} - Log {}", thread_message, i);
                match level {
                    SlogLevel::Critical => slog::crit!(&thread_logger, "{}", log_message),
                    SlogLevel::Error => slog::error!(&thread_logger, "{}", log_message),
                    SlogLevel::Warning => slog::warn!(&thread_logger, "{}", log_message),
                    SlogLevel::Info => slog::info!(&thread_logger, "{}", log_message),
                    SlogLevel::Debug => slog::debug!(&thread_logger, "{}", log_message),
                    SlogLevel::Trace => slog::trace!(&thread_logger, "{}", log_message),
                }
            }
        });

        handles.push(handle);
    }

    // Start all threads simultaneously
    barrier.wait();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
}

fn bench_tracing_concurrent(
    level: TracingLevel,
    message: &str,
    thread_count: usize,
    logs_per_thread: usize,
) {
    let barrier = Arc::new(Barrier::new(thread_count + 1));
    let mut handles = Vec::with_capacity(thread_count);

    for thread_id in 0..thread_count {
        let thread_barrier = Arc::clone(&barrier);
        let thread_message = format!("Thread {} - {}", thread_id, message);

        let handle = thread::spawn(move || {
            // Wait for all threads to be ready
            thread_barrier.wait();

            // Log messages at the specified level
            for i in 0..logs_per_thread {
                let log_message = format!("{} - Log {}", thread_message, i);
                match level {
                    TracingLevel::ERROR => tracing::error!("{}", log_message),
                    TracingLevel::WARN => tracing::warn!("{}", log_message),
                    TracingLevel::INFO => tracing::info!("{}", log_message),
                    TracingLevel::DEBUG => tracing::debug!("{}", log_message),
                    TracingLevel::TRACE => tracing::trace!("{}", log_message),
                }
            }
        });

        handles.push(handle);
    }

    // Start all threads simultaneously
    barrier.wait();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
}

fn bench_loguru_concurrent(
    logger: &Logger,
    level: LoguruLogLevel,
    message: &str,
    thread_count: usize,
    logs_per_thread: usize,
) {
    let logger = Arc::new(logger.clone());
    let barrier = Arc::new(Barrier::new(thread_count + 1));
    let mut handles = Vec::with_capacity(thread_count);

    for thread_id in 0..thread_count {
        let thread_logger = Arc::clone(&logger);
        let thread_barrier = Arc::clone(&barrier);
        let thread_message = format!("Thread {} - {}", thread_id, message);

        let handle = thread::spawn(move || {
            thread_barrier.wait();
            for i in 0..logs_per_thread {
                let log_message = format!("{} - Log {}", thread_message, i);
                match level {
                    LoguruLogLevel::Critical => {
                        thread_logger.log_message(rust_loguru::LogLevel::Critical, &log_message)
                    }
                    LoguruLogLevel::Error => thread_logger.error(&log_message),
                    LoguruLogLevel::Warning => thread_logger.warn(&log_message),
                    LoguruLogLevel::Success => {
                        thread_logger.log_message(rust_loguru::LogLevel::Success, &log_message)
                    }
                    LoguruLogLevel::Info => thread_logger.info(&log_message),
                    LoguruLogLevel::Debug => thread_logger.debug(&log_message),
                    LoguruLogLevel::Trace => {
                        thread_logger.log_message(rust_loguru::LogLevel::Trace, &log_message)
                    }
                };
            }
        });

        handles.push(handle);
    }

    // Start all threads simultaneously
    barrier.wait();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
}

fn benchmark_concurrent_logging(c: &mut Criterion) {
    // Define concurrency scenarios
    let thread_counts = [2, 4, 8, 16];
    let logs_per_thread = 100; // Fixed number of logs per thread
    let message_size = 50; // Fixed message size

    // Create test message
    let message = generate_message(message_size);

    // Define log levels to test
    let log_levels = [(
        "INFO",
        LogLevel::Info,
        SlogLevel::Info,
        TracingLevel::INFO,
        LoguruLogLevel::Info,
    )];

    // Create benchmark group
    let mut group = c.benchmark_group("concurrent_logging");
    group.sampling_mode(SamplingMode::Flat);
    group.measurement_time(Duration::from_secs(20));

    // Setup loggers
    setup_log(LogLevel::Info);
    let slog_logger = setup_slog(SlogLevel::Info);
    setup_tracing(TracingLevel::INFO);
    let loguru_logger = setup_loguru(LoguruLogLevel::Info);

    // Run benchmarks for each thread count
    for &thread_count in thread_counts.iter() {
        for (level_name, log_level, slog_level, tracing_level, loguru_level) in log_levels.iter() {
            // Set throughput to indicate the total number of log operations
            let total_logs = thread_count as u64 * logs_per_thread as u64;
            group.throughput(Throughput::Elements(total_logs));

            // Benchmark log crate
            let benchmark_id = format!("log/{}_{}_threads", level_name, thread_count);
            group.bench_function(BenchmarkId::new("log", &benchmark_id), |b| {
                b.iter(|| {
                    bench_log_concurrent(*log_level, &message, thread_count, logs_per_thread)
                });
            });

            // Benchmark slog
            let benchmark_id = format!("slog/{}_{}_threads", level_name, thread_count);
            group.bench_function(BenchmarkId::new("slog", &benchmark_id), |b| {
                b.iter(|| {
                    bench_slog_concurrent(
                        &slog_logger,
                        *slog_level,
                        &message,
                        thread_count,
                        logs_per_thread,
                    )
                });
            });

            // Benchmark tracing
            let benchmark_id = format!("tracing/{}_{}_threads", level_name, thread_count);
            group.bench_function(BenchmarkId::new("tracing", &benchmark_id), |b| {
                b.iter(|| {
                    bench_tracing_concurrent(
                        *tracing_level,
                        &message,
                        thread_count,
                        logs_per_thread,
                    )
                });
            });

            // Benchmark loguru
            let benchmark_id = format!("loguru/{}_{}_threads", level_name, thread_count);
            group.bench_function(BenchmarkId::new("loguru", &benchmark_id), |b| {
                b.iter(|| {
                    bench_loguru_concurrent(
                        &loguru_logger,
                        *loguru_level,
                        &message,
                        thread_count,
                        logs_per_thread,
                    )
                });
            });
        }
    }

    group.finish();
}

criterion_group!(
    name = concurrent_benches;
    config = Criterion::default().sample_size(15);
    targets = benchmark_concurrent_logging
);
criterion_main!(concurrent_benches);
