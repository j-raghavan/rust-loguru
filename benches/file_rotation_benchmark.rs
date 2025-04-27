use criterion::{
    criterion_group, criterion_main, BenchmarkId, Criterion, SamplingMode, Throughput,
};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Once;
use std::time::Duration;

// Import libraries to benchmark
use log::{Level as LogLevel, LevelFilter};
use rust_loguru::{Logger, LogLevel as LoguruLogLevel};
use slog::{Drain, Level as SlogLevel, Logger as SlogLogger};
use tracing::{Level as TracingLevel};

// Constants for benchmark configuration
const LOG_DIR: &str = "./benchmark_logs";
const LOG_FILE_SIZE: usize = 1024 * 1024; // 1MB
const LOGS_PER_TEST: usize = 10_000;
const MESSAGE_SIZE: usize = 200;

// Ensure clean directory state between benchmarks
static CLEANUP_INIT: Once = Once::new();

// Helper function to generate test messages
fn generate_message(size: usize) -> String {
    let base = "File rotation benchmark test message with timestamp: ";
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
    let mut message = format!("{}{}", base, timestamp);
    
    if size <= message.len() {
        return message[0..size].to_string();
    }
    
    // Pad to the desired size
    while message.len() < size {
        message.push_str(" [padding]");
    }
    message.truncate(size);
    message
}

// Create and manage log directories
fn setup_log_dir() -> io::Result<()> {
    let path = Path::new(LOG_DIR);
    
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    
    fs::create_dir_all(path)?;
    Ok(())
}

fn get_log_file_path(prefix: &str) -> PathBuf {
    let mut path = PathBuf::from(LOG_DIR);
    path.push(format!("{}.log", prefix));
    path
}

fn cleanup_logs() {
    CLEANUP_INIT.call_once(|| {
        let _ = setup_log_dir();
    });
}

// Setup loggers with file output and rotation capabilities
fn setup_log4rs() -> io::Result<()> {
    use log4rs::append::rolling_file::{
        policy::compound::{
            roll::fixed_window::FixedWindowRoller,
            trigger::size::SizeTrigger,
            CompoundPolicy,
        },
        RollingFileAppender,
    };
    use log4rs::config::{Appender, Config, Root};
    use log4rs::encode::pattern::PatternEncoder;

    let log_file = get_log_file_path("log4rs");
    
    // Create a roller that will keep up to 5 archived log files
    let roller = FixedWindowRoller::builder()
        .build(&format!("{}/log4rs.{{}}.log", LOG_DIR), 5)
        .unwrap();

    // Create a size-based triggering policy
    let trigger = SizeTrigger::new(LOG_FILE_SIZE as u64);
    
    // Create a compound policy that uses the trigger and roller
    let policy = CompoundPolicy::new(Box::new(trigger), Box::new(roller));
    
    // Create a rolling file appender with the compound policy
    let appender = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build(log_file, Box::new(policy))
        .unwrap();
    
    // Build a config with the appender
    let config = Config::builder()
        .appender(Appender::builder().build("rolling", Box::new(appender)))
        .build(Root::builder().appender("rolling").build(LevelFilter::Info))
        .unwrap();
    
    // Initialize the logger with the config
    let _ = log4rs::init_config(config).unwrap();
    
    Ok(())
}

fn setup_slog_with_rotation() -> io::Result<SlogLogger> {
    let log_file = get_log_file_path("slog");
    
    // Create a file for slog to write to
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)?;
    
    // Create a decorator and drain
    let decorator = slog_term::PlainDecorator::new(file);
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    
    // slog doesn't have built-in rotation, so we'll simulate it in the benchmark function
    
    Ok(SlogLogger::root(drain, slog::o!()))
}

fn setup_tracing_with_rotation() -> io::Result<()> {
    use tracing_appender::rolling::{RollingFileAppender, Rotation};
    use tracing_subscriber::fmt::writer::MakeWriterExt;
    
    // Create a rolling file appender that rolls daily
    let file_appender = RollingFileAppender::new(
        Rotation::NEVER, // We'll simulate rotation in the benchmark function
        LOG_DIR,
        "tracing.log",
    );
    
    // Set up the subscriber with the rolling file appender
    tracing_subscriber::fmt()
        .with_writer(file_appender.with_max_level(TracingLevel::INFO))
        .with_ansi(false)
        .init();
    
    Ok(())
}

fn setup_loguru_with_rotation() -> io::Result<Logger> {
    let log_file = get_log_file_path("loguru");
    
    // Create a logger with file rotation
    let logger = Logger::new()
        .add_handler(
            rust_loguru::handlers::file::File::new(log_file.to_str().unwrap())
                .set_rotation_policy(rust_loguru::handlers::file::RotationPolicy::Size(LOG_FILE_SIZE))
                .set_retention_policy(rust_loguru::handlers::file::RetentionPolicy::Count(5))
        )
        .set_level(LoguruLogLevel::INFO);
    
    Ok(logger)
}

// Benchmark functions with file rotation
fn bench_log4rs_with_rotation(message: &str) -> io::Result<()> {
    // Log enough messages to trigger rotation
    for i in 0..LOGS_PER_TEST {
        let log_message = format!("[{}] {}", i, message);
        log::info!("{}", log_message);
    }
    
    Ok(())
}

fn bench_slog_with_rotation(logger: &SlogLogger, message: &str) -> io::Result<()> {
    // Log enough messages to trigger rotation
    for i in 0..LOGS_PER_TEST {
        let log_message = format!("[{}] {}", i, message);
        slog::info!(logger, "{}", log_message);
    }
    
    // slog doesn't have built-in rotation, so let's simulate it here
    // by checking the file size and rotating manually
    let log_file = get_log_file_path("slog");
    
    if let Ok(metadata) = fs::metadata(&log_file) {
        if metadata.len() > LOG_FILE_SIZE as u64 {
            // Rotate the log file
            for i in (1..5).rev() {
                let from = get_log_file_path(&format!("slog.{}", i));
                let to = get_log_file_path(&format!("slog.{}", i + 1));
                
                if from.exists() {
                    let _ = fs::rename(from, to);
                }
            }
            
            let to = get_log_file_path("slog.1");
            if log_file.exists() {
                let _ = fs::rename(&log_file, to);
            }
            
            // Create a new log file
            let _ = File::create(&log_file);
        }
    }
    
    Ok(())
}

fn bench_tracing_with_rotation(message: &str) -> io::Result<()> {
    // Log enough messages to trigger rotation
    for i in 0..LOGS_PER_TEST {
        let log_message = format!("[{}] {}", i, message);
        tracing::info!("{}", log_message);
    }
    
    // tracing_appender handles rotation automatically based on the Rotation policy
    // For this benchmark, we're using NEVER, so we'll simulate it here similar to slog
    let log_file = PathBuf::from(LOG_DIR).join("tracing.log");
    
    if let Ok(metadata) = fs::metadata(&log_file) {
        if metadata.len() > LOG_FILE_SIZE as u64 {
            // Rotate the log file
            for i in (1..5).rev() {
                let from = PathBuf::from(LOG_DIR).join(format!("tracing.{}.log", i));
                let to = PathBuf::from(LOG_DIR).join(format!("tracing.{}.log", i + 1));
                
                if from.exists() {
                    let _ = fs::rename(from, to);
                }
            }
            
            let to = PathBuf::from(LOG_DIR).join("tracing.1.log");
            if log_file.exists() {
                let _ = fs::rename(&log_file, to);
            }
            
            // Create a new log file
            let _ = File::create(&log_file);
        }
    }
    
    Ok(())
}

fn bench_loguru_with_rotation(logger: &Logger, message: &str) -> io::Result<()> {
    // Log enough messages to trigger rotation
    for i in 0..LOGS_PER_TEST {
        let log_message = format!("[{}] {}", i, message);
        logger.info(&log_message);
    }
    
    // rust-loguru should handle the rotation automatically based on its configuration
    
    Ok(())
}

fn benchmark_file_rotation(c: &mut Criterion) {
    // Clean up any existing logs
    cleanup_logs();
    
    // Generate a test message
    let message = generate_message(MESSAGE_SIZE);
    
    // Create benchmark group
    let mut group = c.benchmark_group("file_rotation");
    group.sampling_mode(SamplingMode::Flat);
    group.measurement_time(Duration::from_secs(15));
    group.throughput(Throughput::Elements(LOGS_PER_TEST as u64));
    
    // Run the benchmarks
    // Note: For each benchmark, we'll reset the log directory first
    
    // log4rs benchmark
    {
        group.bench_function(BenchmarkId::new("log4rs", "file_rotation"), |b| {
            b.iter_custom(|iters| {
                let mut total_duration = Duration::new(0, 0);
                
                for _ in 0..iters {
                    let _ = setup_log_dir();
                    let _ = setup_log4rs();
                    
                    let start = std::time::Instant::now();
                    let _ = bench_log4rs_with_rotation(&message);
                    total_duration += start.elapsed();
                }
                
                total_duration
            });
        });
    }
    
    // slog benchmark
    {
        group.bench_function(BenchmarkId::new("slog", "file_rotation"), |b| {
            b.iter_custom(|iters| {
                let mut total_duration = Duration::new(0, 0);
                
                for _ in 0..iters {
                    let _ = setup_log_dir();
                    let logger = setup_slog_with_rotation().unwrap();
                    
                    let start = std::time::Instant::now();
                    let _ = bench_slog_with_rotation(&logger, &message);
                    total_duration += start.elapsed();
                }
                
                total_duration
            });
        });
    }
    
    // tracing benchmark
    {
        group.bench_function(BenchmarkId::new("tracing", "file_rotation"), |b| {
            b.iter_custom(|iters| {
                let mut total_duration = Duration::new(0, 0);
                
                for _ in 0..iters {
                    let _ = setup_log_dir();
                    let _ = setup_tracing_with_rotation();
                    
                    let start = std::time::Instant::now();
                    let _ = bench_tracing_with_rotation(&message);
                    total_duration += start.elapsed();
                }
                
                total_duration
            });
        });
    }
    
    // loguru benchmark
    {
        group.bench_function(BenchmarkId::new("loguru", "file_rotation"), |b| {
            b.iter_custom(|iters| {
                let mut total_duration = Duration::new(0, 0);
                
                for _ in 0..iters {
                    let _ = setup_log_dir();
                    let logger = setup_loguru_with_rotation().unwrap();
                    
                    let start = std::time::Instant::now();
                    let _ = bench_loguru_with_rotation(&logger, &message);
                    total_duration += start.elapsed();
                }
                
                total_duration
            });
        });
    }
    
    group.finish();
    
    // Final cleanup
    let _ = fs::remove_dir_all(LOG_DIR);
}

criterion_group!(
    name = file_rotation_benches;
    config = Criterion::default().sample_size(10);
    targets = benchmark_file_rotation
);
criterion_main!(file_rotation_benches);
