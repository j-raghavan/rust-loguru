use criterion::{
    criterion_group, criterion_main, BenchmarkId, Criterion, SamplingMode, Throughput,
};
use std::io::sink;
use std::time::Duration;

// Import the libraries we're benchmarking
use rust_loguru::handler::{new_handler_ref, NullHandler};
use rust_loguru::{LogLevel as LoguruLogLevel, Logger};
use slog::{Drain, Level as SlogLevel, Logger as SlogLogger};

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

// Setup functions for each logger
fn setup_slog(level: SlogLevel) -> SlogLogger {
    let drain = slog::Discard.fuse();
    let drain = drain.filter_level(level).fuse();
    SlogLogger::root(drain, slog::o!())
}

fn setup_loguru(level: LoguruLogLevel) -> Logger {
    let mut logger = Logger::new(level.into());
    logger.add_handler(new_handler_ref(NullHandler::new(level.into())));
    logger
}

// Benchmark functions
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

fn benchmark_slog_vs_loguru(c: &mut Criterion) {
    // Define test parameters
    let message_sizes = [10, 100, 500, 1000, 5000];
    let log_levels = [
        ("CRITICAL", SlogLevel::Critical, LoguruLogLevel::Critical),
        ("ERROR", SlogLevel::Error, LoguruLogLevel::Error),
        ("WARNING", SlogLevel::Warning, LoguruLogLevel::Warning),
        ("INFO", SlogLevel::Info, LoguruLogLevel::Info),
        ("DEBUG", SlogLevel::Debug, LoguruLogLevel::Debug),
        ("TRACE", SlogLevel::Trace, LoguruLogLevel::Trace),
    ];

    // Create benchmark group
    let mut group = c.benchmark_group("slog_vs_loguru");
    group.sampling_mode(SamplingMode::Flat);
    group.measurement_time(Duration::from_secs(5));

    // Setup loggers with maximum verbosity
    let slog_logger = setup_slog(SlogLevel::Trace);
    let loguru_logger = setup_loguru(LoguruLogLevel::Trace);

    // Run benchmarks for each combination
    for size in message_sizes.iter() {
        let message = generate_message(*size);

        for (level_name, slog_level, loguru_level) in log_levels.iter() {
            group.throughput(Throughput::Elements(1));

            // Benchmark slog
            let benchmark_id = format!("slog/{}_{}", level_name, size);
            group.bench_with_input(
                BenchmarkId::new("slog", &benchmark_id),
                &message,
                |b, msg| {
                    b.iter(|| bench_slog(&slog_logger, *slog_level, msg));
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
    name = slog_vs_loguru_benches;
    config = Criterion::default().sample_size(100);
    targets = benchmark_slog_vs_loguru
);
criterion_main!(slog_vs_loguru_benches);
