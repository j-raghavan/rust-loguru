use criterion::{
    criterion_group, criterion_main, BenchmarkId, Criterion, SamplingMode, Throughput,
};
use std::io::sink;
use std::time::Duration;
use std::collections::HashMap;

// Import libraries to benchmark
use log::{Level as LogLevel, LevelFilter};
use rust_loguru::{Logger, LogLevel as LoguruLogLevel};
use slog::{Drain, Level as SlogLevel, Logger as SlogLogger, o, OwnedKV};
use tracing::{Level as TracingLevel};
use serde_json::{json, Value as JsonValue};

// Helper function to generate test data
fn generate_structured_data(num_fields: usize, field_size: usize) -> (JsonValue, String, HashMap<String, String>) {
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
        .filter_module("structured_logging_benchmark", match level {
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        })
        .target(env_logger::Target::Pipe(Box::new(sink())))
        .is_test(true)
        .try_init();
}

fn setup_slog(level: SlogLevel) -> SlogLogger {
    let decorator = slog_term::PlainDecorator::new(Box::new(sink()) as Box<dyn std::io::Write>);
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = drain.filter_level(level).fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
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
    Logger::new()
        .add_handler(rust_loguru::handlers::null::Null::new())
        .set_level(level)
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

fn bench_slog_structured(logger: &SlogLogger, level: SlogLevel, message: &str, fields: &HashMap<String, String>) {
    // Create a dynamic key-value list for slog
    let mut kv = vec![];
    for (k, v) in fields {
        kv.push((k.as_str(), v.as_str()));
    }
    
    // slog has native support for structured logging with key-value pairs
    let owned_kv = OwnedKV(o!(kv.remove(0).0 => kv.remove(0).1));
    
    match level {
        SlogLevel::Critical => slog::crit!(logger, "{}", message; owned_kv),
        SlogLevel::Error => slog::error!(logger, "{}", message; owned_kv),
        SlogLevel::Warning => slog::warn!(logger, "{}", message; owned_kv),
        SlogLevel::Info => slog::info!(logger, "{}", message; owned_kv),
        SlogLevel::Debug => slog::debug!(logger, "{}", message; owned_kv),
        SlogLevel::Trace => slog::trace!(logger, "{}", message; owned_kv),
    }
}

fn bench_tracing_structured(level: TracingLevel, message: &str, fields: &HashMap<String, String>) {
    // tracing has native support for structured fields
    match level {
        TracingLevel::ERROR => {
            let span = tracing::span!(TracingLevel::ERROR, "structured_log", message = message);
            let _guard = span.enter();
            for (k, v) in fields {
                tracing::event!(TracingLevel::ERROR, {k} = v, "{}", message);
            }
        },
        TracingLevel::WARN => {
            let span = tracing::span!(TracingLevel::WARN, "structured_log", message = message);
            let _guard = span.enter();
            for (k, v) in fields {
                tracing::event!(TracingLevel::WARN, {k} = v, "{}", message);
            }
        },
        TracingLevel::INFO => {
            let span = tracing::span!(TracingLevel::INFO, "structured_log", message = message);
            let _guard = span.enter();
            for (k, v) in fields {
                tracing::event!(TracingLevel::INFO, {k} = v, "{}", message);
            }
        },
        TracingLevel::DEBUG => {
            let span = tracing::span!(TracingLevel::DEBUG, "structured_log", message = message);
            let _guard = span.enter();
            for (k, v) in fields {
                tracing::event!(TracingLevel::DEBUG, {k} = v, "{}", message);
            }
        },
        TracingLevel::TRACE => {
            let span = tracing::span!(TracingLevel::TRACE, "structured_log", message = message);
            let _guard = span.enter();
            for (k, v) in fields {
                tracing::event!(TracingLevel::TRACE, {k} = v, "{}", message);
            }
        },
    }
}

fn bench_loguru_structured(logger: &Logger, level: LoguruLogLevel, message: &str, json_value: &JsonValue) {
    // Assuming loguru has methods for structured logging
    // Adjust this to match your actual API
    match level {
        LoguruLogLevel::CRITICAL => {
            // If your API supports structured JSON directly:
            // logger.critical_with_fields(message, json_value);
            
            // Or if your API supports a record builder pattern:
            logger.critical(&format!("{} {}", message, json_value))
        },
        LoguruLogLevel::ERROR => {
            logger.error(&format!("{} {}", message, json_value))
        },
        LoguruLogLevel::WARNING => {
            logger.warning(&format!("{} {}", message, json_value))
        },
        LoguruLogLevel::SUCCESS => {
            logger.success(&format!("{} {}", message, json_value))
        },
        LoguruLogLevel::INFO => {
            logger.info(&format!("{} {}", message, json_value))
        },
        LoguruLogLevel::DEBUG => {
            logger.debug(&format!("{} {}", message, json_value))
        },
        LoguruLogLevel::TRACE => {
            logger.trace(&format!("{} {}", message, json_value))
        },
    }
}

fn benchmark_structured_logging(c: &mut Criterion) {
    // Define different numbers of structured fields to test
    let field_counts = [5, 20, 50];
    let field_size = 10; // Size of each field value
    
    // Define log levels to test
    let log_levels = [
        (
            "INFO", 
            LogLevel::Info, 
            SlogLevel::Info, 
            TracingLevel::INFO, 
            LoguruLogLevel::INFO
        ),
    ];
    
    let mut group = c.benchmark_group("structured_logging");
    group.sampling_mode(SamplingMode::Flat);
    group.measurement_time(Duration::from_secs(10));
    
    // Setup loggers
    setup_log(LogLevel::Trace);
    let slog_logger = setup_slog(SlogLevel::Trace);
    setup_tracing(TracingLevel::TRACE);
    let loguru_logger = setup_loguru(LoguruLogLevel::TRACE);
    
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
                    b.iter(|| bench_loguru_structured(&loguru_logger, *loguru_level, message, json_val));
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
