use criterion::{
    criterion_group, criterion_main, BenchmarkId, Criterion, SamplingMode, Throughput,
};
use std::io::sink;
use std::time::Duration;

// Import libraries to benchmark
use log::{Level as LogLevel, LevelFilter};
use rust_loguru::handler::{new_handler_ref, NullHandler};
use rust_loguru::{LogLevel as LoguruLogLevel, Logger};
use slog::{Drain, Level as SlogLevel, Logger as SlogLogger};
use tracing::Level as TracingLevel;

// Message generator for creating test messages of different sizes
fn generate_message(size: usize) -> String {
    let base = "This is a test log message for benchmarking.";
    if size <= base.len() {
        return base[0..size].to_string();
    }

    let mut message = base.to_string();
    while message.len() < size {
        message.push_str(" Additional padding for benchmark.");
    }
    message[0..size].to_string()
}

// Setup loggers
fn setup_log(level: LogLevel) {
    let _ = env_logger::Builder::new()
        .filter_level(LevelFilter::Trace)
        .filter_module(
            "basic_logging_benchmark",
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
}

fn setup_slog(level: SlogLevel) -> SlogLogger {
    let drain = slog::Discard.fuse();
    let drain = drain.filter_level(level).fuse();
    SlogLogger::root(drain, slog::o!())
}

fn setup_tracing(level: TracingLevel) {
    let _ = tracing_subscriber::fmt()
        .with_max_level(level)
        .with_writer(sink)
        .with_ansi(false)
        .try_init();
}

fn setup_loguru(level: LoguruLogLevel) -> Logger {
    let mut logger = Logger::new(level.into());
    logger.add_handler(new_handler_ref(NullHandler::new(level.into())));
    logger
}

// Benchmark functions for each library
fn bench_log(level: LogLevel, message: &str) {
    match level {
        LogLevel::Error => log::error!("{}", message),
        LogLevel::Warn => log::warn!("{}", message),
        LogLevel::Info => log::info!("{}", message),
        LogLevel::Debug => log::debug!("{}", message),
        LogLevel::Trace => log::trace!("{}", message),
    }
}

fn bench_slog(logger: &SlogLogger, level: SlogLevel, message: &str) {
    match level {
        SlogLevel::Critical => slog::crit!(logger, "{}", message),
        SlogLevel::Error => slog::error!(logger, "{}", message),
        SlogLevel::Warning => slog::warn!(logger, "{}", message),
        SlogLevel::Info => slog::info!(logger, "{}", message),
        SlogLevel::Debug => slog::debug!(logger, "{}", message),
        SlogLevel::Trace => slog::trace!(logger, "{}", message),
    }
}

fn bench_tracing(level: TracingLevel, message: &str) {
    match level {
        TracingLevel::ERROR => tracing::error!("{}", message),
        TracingLevel::WARN => tracing::warn!("{}", message),
        TracingLevel::INFO => tracing::info!("{}", message),
        TracingLevel::DEBUG => tracing::debug!("{}", message),
        TracingLevel::TRACE => tracing::trace!("{}", message),
    }
}

fn bench_loguru(logger: &Logger, level: LoguruLogLevel, message: &str) {
    match level {
        LoguruLogLevel::Critical => logger.log_message(rust_loguru::LogLevel::Critical, message),
        LoguruLogLevel::Error => logger.error(message),
        LoguruLogLevel::Warning => logger.warn(message),
        LoguruLogLevel::Success => logger.log_message(rust_loguru::LogLevel::Success, message),
        LoguruLogLevel::Info => logger.info(message),
        LoguruLogLevel::Debug => logger.debug(message),
        LoguruLogLevel::Trace => logger.log_message(rust_loguru::LogLevel::Trace, message),
    };
}

fn benchmark_basic_logging(c: &mut Criterion) {
    // Define message sizes to test
    let message_sizes = [10, 100, 1000];

    // Define log levels to test
    let log_levels = [
        (
            "ERROR",
            LogLevel::Error,
            SlogLevel::Error,
            TracingLevel::ERROR,
            LoguruLogLevel::Error,
        ),
        (
            "INFO",
            LogLevel::Info,
            SlogLevel::Info,
            TracingLevel::INFO,
            LoguruLogLevel::Info,
        ),
        (
            "DEBUG",
            LogLevel::Debug,
            SlogLevel::Debug,
            TracingLevel::DEBUG,
            LoguruLogLevel::Debug,
        ),
    ];

    // Create benchmark group
    let mut group = c.benchmark_group("basic_logging");
    group.sampling_mode(SamplingMode::Flat);
    group.measurement_time(Duration::from_secs(5));

    // Setup loggers
    setup_log(LogLevel::Trace);
    let slog_logger = setup_slog(SlogLevel::Trace);
    setup_tracing(TracingLevel::TRACE);
    let loguru_logger = setup_loguru(LoguruLogLevel::Trace);

    // Run benchmarks for each message size and log level combination
    for size in message_sizes.iter() {
        let message = generate_message(*size);

        for (level_name, log_level, slog_level, tracing_level, loguru_level) in log_levels.iter() {
            group.throughput(Throughput::Elements(1));

            // Benchmark log crate
            let benchmark_id = format!("log/{}_{}", level_name, size);
            group.bench_with_input(
                BenchmarkId::new("log", &benchmark_id),
                &message,
                |b, msg| {
                    b.iter(|| bench_log(*log_level, msg));
                },
            );

            // Benchmark slog
            let benchmark_id = format!("slog/{}_{}", level_name, size);
            group.bench_with_input(
                BenchmarkId::new("slog", &benchmark_id),
                &message,
                |b, msg| {
                    b.iter(|| bench_slog(&slog_logger, *slog_level, msg));
                },
            );

            // Benchmark tracing
            let benchmark_id = format!("tracing/{}_{}", level_name, size);
            group.bench_with_input(
                BenchmarkId::new("tracing", &benchmark_id),
                &message,
                |b, msg| {
                    b.iter(|| bench_tracing(*tracing_level, msg));
                },
            );

            // Benchmark loguru
            let benchmark_id = format!("loguru/{}_{}", level_name, size);
            group.bench_with_input(
                BenchmarkId::new("loguru", &benchmark_id),
                &message,
                |b, msg| {
                    b.iter(|| bench_loguru(&loguru_logger, *loguru_level, msg));
                },
            );
        }
    }

    group.finish();
}

criterion_group!(
    name = basic_logging_benches;
    config = Criterion::default().sample_size(100);
    targets = benchmark_basic_logging
);
criterion_main!(basic_logging_benches);
