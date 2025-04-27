use criterion::{
    criterion_group, criterion_main, BenchmarkId, Criterion, SamplingMode, Throughput,
};
use std::io::sink;
use std::sync::Once;
use std::time::Duration;

// Import libraries to benchmark
use log::{Level as LogLevel, LevelFilter};
use rust_loguru::{Logger, LogLevel as LoguruLogLevel};
use slog::{Drain, Level as SlogLevel, Logger as SlogLogger};
use tracing::{Level as TracingLevel};

// Ensure loggers are only initialized once
static LOG_INIT: Once = Once::new();
static TRACING_INIT: Once = Once::new();

// Helper function to generate test messages
fn generate_message(size: usize) -> String {
    let base = "High volume logging benchmark test message. ";
    
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
            .filter_module("high_volume_benchmark", match level {
                LogLevel::Error => LevelFilter::Error,
                LogLevel::Warn => LevelFilter::Warn,
                LogLevel::Info => LevelFilter::Info,
                LogLevel::Debug => LevelFilter::Debug,
                LogLevel::Trace => LevelFilter::Trace,
            })
            .target(env_logger::Target::Pipe(Box::new(sink())))
            .is_test(true)
            .try_init();
    });
}

fn setup_slog(level: SlogLevel) -> SlogLogger {
    let decorator = slog_term::PlainDecorator::new(Box::new(sink()) as Box<dyn std::io::Write>);
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = drain.filter_level(level).fuse();
    let drain = slog_async::Async::new(drain)
        .build()
        .fuse();
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
    Logger::new()
        .add_handler(rust_loguru::handlers::null::Null::new())
        .set_level(level)
}

// Benchmark functions for each library
fn bench_log_high_volume(level: LogLevel, message: &str, count: usize) {
    for _ in 0..count {
        match level {
            LogLevel::Error => log::error!("{}", message),
            LogLevel::Warn => log::warn!("{}", message),
            LogLevel::Info => log::info!("{}", message),
            LogLevel::Debug => log::debug!("{}", message),
            LogLevel::Trace => log::trace!("{}", message),
        }
    }
}

fn bench_slog_high_volume(logger: &SlogLogger, level: SlogLevel, message: &str, count: usize) {
    for _ in 0..count {
        match level {
            SlogLevel::Critical => slog::crit!(logger, "{}", message),
            SlogLevel::Error => slog::error!(logger, "{}", message),
            SlogLevel::Warning => slog::warn!(logger, "{}", message),
            SlogLevel::Info => slog::info!(logger, "{}", message),
            SlogLevel::Debug => slog::debug!(logger, "{}", message),
            SlogLevel::Trace => slog::trace!(logger, "{}", message),
        }
    }
}

fn bench_tracing_high_volume(level: TracingLevel, message: &str, count: usize) {
    for _ in 0..count {
        match level {
            TracingLevel::ERROR => tracing::error!("{}", message),
            TracingLevel::WARN => tracing::warn!("{}", message),
            TracingLevel::INFO => tracing::info!("{}", message),
            TracingLevel::DEBUG => tracing::debug!("{}", message),
            TracingLevel::TRACE => tracing::trace!("{}", message),
        }
    }
}

fn bench_loguru_high_volume(logger: &Logger, level: LoguruLogLevel, message: &str, count: usize) {
    for _ in 0..count {
        match level {
            LoguruLogLevel::CRITICAL => logger.critical(message),
            LoguruLogLevel::ERROR => logger.error(message),
            LoguruLogLevel::WARNING => logger.warning(message),
            LoguruLogLevel::SUCCESS => logger.success(message),
            LoguruLogLevel::INFO => logger.info(message),
            LoguruLogLevel::DEBUG => logger.debug(message),
            LoguruLogLevel::TRACE => logger.trace(message),
        }
    }
}

fn benchmark_high_volume(c: &mut Criterion) {
    // Define high volume scenarios
    let log_counts = [100, 1_000, 10_000];
    let message_size = 100; // Fixed message size
    
    // Create message
    let message = generate_message(message_size);
    
    // Define log levels to test (we'll focus on INFO level for high volume)
    let log_levels = [
        (
            "INFO", 
            LogLevel::Info, 
            SlogLevel::Info, 
            TracingLevel::INFO, 
            LoguruLogLevel::INFO
        ),
    ];
    
    // Create benchmark group
    let mut group = c.benchmark_group("high_volume_logging");
    group.sampling_mode(SamplingMode::Flat);
    group.measurement_time(Duration::from_secs(20));
    
    // Setup loggers
    setup_log(LogLevel::Info);
    let slog_logger = setup_slog(SlogLevel::Info);
    setup_tracing(TracingLevel::INFO);
    let loguru_logger = setup_loguru(LoguruLogLevel::INFO);
    
    // Run benchmarks for each count and log level combination
    for &count in log_counts.iter() {
        for (level_name, log_level, slog_level, tracing_level, loguru_level) in log_levels.iter() {
            // Set throughput to indicate the number of log operations
            group.throughput(Throughput::Elements(count as u64));
            
            // Benchmark log crate
            let benchmark_id = format!("log/{}_{}", level_name, count);
            group.bench_function(
                BenchmarkId::new("log", &benchmark_id), 
                |b| {
                    b.iter(|| bench_log_high_volume(*log_level, &message, count));
                },
            );
            
            // Benchmark slog
            let benchmark_id = format!("slog/{}_{}", level_name, count);
            group.bench_function(
                BenchmarkId::new("slog", &benchmark_id), 
                |b| {
                    b.iter(|| bench_slog_high_volume(&slog_logger, *slog_level, &message, count));
                },
            );
            
            // Benchmark tracing
            let benchmark_id = format!("tracing/{}_{}", level_name, count);
            group.bench_function(
                BenchmarkId::new("tracing", &benchmark_id), 
                |b| {
                    b.iter(|| bench_tracing_high_volume(*tracing_level, &message, count));
                },
            );
            
            // Benchmark loguru
            let benchmark_id = format!("loguru/{}_{}", level_name, count);
            group.bench_function(
                BenchmarkId::new("loguru", &benchmark_id), 
                |b| {
                    b.iter(|| bench_loguru_high_volume(&loguru_logger, *loguru_level, &message, count));
                },
            );
        }
    }
    
    group.finish();
}

criterion_group!(
    name = high_volume_benches;
    config = Criterion::default().sample_size(20);
    targets = benchmark_high_volume
);
criterion_main!(high_volume_benches);
