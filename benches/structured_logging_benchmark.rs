use criterion::{
    criterion_group, criterion_main, BenchmarkId, Criterion, SamplingMode, Throughput,
};
use std::collections::HashMap;
use std::io::sink;
use std::time::Duration;

// Import libraries to benchmark
use log::{Level as LogLevel, LevelFilter};
use rust_loguru::handler::{new_handler_ref, NullHandler};
use rust_loguru::{LogLevel as LoguruLogLevel, Logger};
use serde_json::Value as JsonValue;
use slog::{Drain, Level as SlogLevel, Logger as SlogLogger};
use tracing::Level as TracingLevel;

// Helper function to generate test data
fn generate_structured_data(
    num_fields: usize,
    field_size: usize,
) -> (JsonValue, String, HashMap<String, String>) {
    let mut json_obj = serde_json::Map::new();
    let mut hashmap = HashMap::new();

    for i in 0..num_fields {
        let key = format!("field_{}", i);
        let value = "x".repeat(field_size);

        json_obj.insert(key.clone(), JsonValue::String(value.clone()));
        hashmap.insert(key, value);
    }

    let json_value = JsonValue::Object(json_obj);
    let json_string = json_value.to_string();

    (json_value, json_string, hashmap)
}

// Setup loggers
fn setup_log(level: LogLevel) {
    let _ = env_logger::Builder::new()
        .filter_level(LevelFilter::Trace)
        .filter_module(
            "structured_logging_benchmark",
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
fn bench_log_structured(level: LogLevel, message: &str, json_str: &str) {
    // log crate doesn't have native structured logging, so we'll just log the JSON string
    match level {
        LogLevel::Error => log::error!("{} {}", message, json_str),
        LogLevel::Warn => log::warn!("{} {}", message, json_str),
        LogLevel::Info => log::info!("{} {}", message, json_str),
        LogLevel::Debug => log::debug!("{} {}", message, json_str),
        LogLevel::Trace => log::trace!("{} {}", message, json_str),
    }
}

fn bench_slog_structured(
    logger: &SlogLogger,
    level: SlogLevel,
    message: &str,
    fields: &HashMap<String, String>,
) {
    let fields_str = fields
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join(", ");
    match level {
        SlogLevel::Critical => slog::crit!(logger, "{} {}", message, fields_str),
        SlogLevel::Error => slog::error!(logger, "{} {}", message, fields_str),
        SlogLevel::Warning => slog::warn!(logger, "{} {}", message, fields_str),
        SlogLevel::Info => slog::info!(logger, "{} {}", message, fields_str),
        SlogLevel::Debug => slog::debug!(logger, "{} {}", message, fields_str),
        SlogLevel::Trace => slog::trace!(logger, "{} {}", message, fields_str),
    }
}

fn bench_tracing_structured(level: TracingLevel, message: &str, fields: &HashMap<String, String>) {
    let fields_str = fields
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join(", ");
    match level {
        TracingLevel::ERROR => {
            tracing::event!(TracingLevel::ERROR, message = %message, fields = %fields_str)
        }
        TracingLevel::WARN => {
            tracing::event!(TracingLevel::WARN, message = %message, fields = %fields_str)
        }
        TracingLevel::INFO => {
            tracing::event!(TracingLevel::INFO, message = %message, fields = %fields_str)
        }
        TracingLevel::DEBUG => {
            tracing::event!(TracingLevel::DEBUG, message = %message, fields = %fields_str)
        }
        TracingLevel::TRACE => {
            tracing::event!(TracingLevel::TRACE, message = %message, fields = %fields_str)
        }
    }
}

fn bench_loguru_structured(
    logger: &Logger,
    level: LoguruLogLevel,
    message: &str,
    json_value: &JsonValue,
) {
    match level {
        LoguruLogLevel::Critical => logger.log_message(
            rust_loguru::LogLevel::Critical,
            format!("{} {}", message, json_value),
        ),
        LoguruLogLevel::Error => logger.error(format!("{} {}", message, json_value)),
        LoguruLogLevel::Warning => logger.warn(format!("{} {}", message, json_value)),
        LoguruLogLevel::Success => logger.log_message(
            rust_loguru::LogLevel::Success,
            format!("{} {}", message, json_value),
        ),
        LoguruLogLevel::Info => logger.info(format!("{} {}", message, json_value)),
        LoguruLogLevel::Debug => logger.debug(format!("{} {}", message, json_value)),
        LoguruLogLevel::Trace => logger.log_message(
            rust_loguru::LogLevel::Trace,
            format!("{} {}", message, json_value),
        ),
    };
}

fn benchmark_structured_logging(c: &mut Criterion) {
    // Define different numbers of structured fields to test
    let field_counts = [5, 20, 50];
    let field_size = 10; // Size of each field value

    // Define log levels to test
    let log_levels = [(
        "INFO",
        LogLevel::Info,
        SlogLevel::Info,
        TracingLevel::INFO,
        LoguruLogLevel::Info,
    )];

    let mut group = c.benchmark_group("structured_logging");
    group.sampling_mode(SamplingMode::Flat);
    group.measurement_time(Duration::from_secs(10));

    // Setup loggers
    setup_log(LogLevel::Trace);
    let slog_logger = setup_slog(SlogLevel::Trace);
    setup_tracing(TracingLevel::TRACE);
    let loguru_logger = setup_loguru(LoguruLogLevel::Trace);

    // Run benchmarks for each field count and log level combination
    for field_count in field_counts.iter() {
        let (json_value, json_string, fields) = generate_structured_data(*field_count, field_size);
        let message = "Structured log message";

        for (level_name, log_level, slog_level, tracing_level, loguru_level) in log_levels.iter() {
            group.throughput(Throughput::Elements(1));

            // Benchmark log crate (doesn't have native structured logging)
            let benchmark_id = format!("log/{}_{}", level_name, field_count);
            group.bench_with_input(
                BenchmarkId::new("log", &benchmark_id),
                &json_string,
                |b, json_str| {
                    b.iter(|| bench_log_structured(*log_level, message, json_str));
                },
            );

            // Benchmark slog (has native structured logging)
            let benchmark_id = format!("slog/{}_{}", level_name, field_count);
            group.bench_with_input(
                BenchmarkId::new("slog", &benchmark_id),
                &fields,
                |b, fields| {
                    b.iter(|| bench_slog_structured(&slog_logger, *slog_level, message, fields));
                },
            );

            // Benchmark tracing (has native structured logging)
            let benchmark_id = format!("tracing/{}_{}", level_name, field_count);
            group.bench_with_input(
                BenchmarkId::new("tracing", &benchmark_id),
                &fields,
                |b, fields| {
                    b.iter(|| bench_tracing_structured(*tracing_level, message, fields));
                },
            );

            // Benchmark loguru
            let benchmark_id = format!("loguru/{}_{}", level_name, field_count);
            group.bench_with_input(
                BenchmarkId::new("loguru", &benchmark_id),
                &json_value,
                |b, json_val| {
                    b.iter(|| {
                        bench_loguru_structured(&loguru_logger, *loguru_level, message, json_val)
                    });
                },
            );
        }
    }

    group.finish();
}

criterion_group!(
    name = structured_logging_benches;
    config = Criterion::default().sample_size(50);
    targets = benchmark_structured_logging
);
criterion_main!(structured_logging_benches);
