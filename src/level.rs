use colored::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

/// Represents the severity level of a log message.
/// Levels are ordered from least to most severe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LogLevel {
    /// The lowest level of logging, used for very detailed debugging information.
    Trace,
    /// Used for debugging information that might be useful in diagnosing problems.
    Debug,
    /// Used for informational messages that highlight the progress of the application.
    Info,
    /// Used to indicate successful operations or positive outcomes.
    Success,
    /// Used for potentially harmful situations that might still allow the application to continue running.
    Warning,
    /// Used for error events that might still allow the application to continue running.
    Error,
    /// Used for very severe error events that will presumably lead the application to abort.
    Critical,
}

impl LogLevel {
    /// Returns the string representation of the log level.
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Success => "SUCCESS",
            LogLevel::Warning => "WARNING",
            LogLevel::Error => "ERROR",
            LogLevel::Critical => "CRITICAL",
        }
    }

    /// Returns the numeric value of the log level.
    /// Uses standard logging level values (5, 10, 20, etc.)
    pub fn as_u8(&self) -> u8 {
        match self {
            LogLevel::Trace => 5,
            LogLevel::Debug => 10,
            LogLevel::Info => 20,
            LogLevel::Success => 25,
            LogLevel::Warning => 30,
            LogLevel::Error => 40,
            LogLevel::Critical => 50,
        }
    }

    /// Returns the ANSI color code for this log level.
    pub fn color(&self) -> &'static str {
        match self {
            LogLevel::Trace => "\x1b[37m",    // white
            LogLevel::Debug => "\x1b[34m",    // blue
            LogLevel::Info => "\x1b[32m",     // green
            LogLevel::Success => "\x1b[36m",  // cyan
            LogLevel::Warning => "\x1b[33m",  // yellow
            LogLevel::Error => "\x1b[31m",    // red
            LogLevel::Critical => "\x1b[35m", // purple
        }
    }

    /// Returns the emoji representation of the log level.
    pub fn emoji(&self) -> &'static str {
        match self {
            LogLevel::Trace => "🔍",
            LogLevel::Debug => "🐛",
            LogLevel::Info => "ℹ️",
            LogLevel::Success => "✅",
            LogLevel::Warning => "⚠️",
            LogLevel::Error => "❌",
            LogLevel::Critical => "💥",
        }
    }

    /// Returns the ANSI reset code.
    pub fn reset_color() -> &'static str {
        "\x1b[0m"
    }

    /// Checks if a module path matches the given pattern.
    /// The pattern can be a simple string match or a glob pattern.
    pub fn matches_module_pattern(&self, module_path: &str, pattern: &str) -> bool {
        if pattern.contains('*') {
            // Simple glob pattern matching
            let pattern = pattern.replace('*', ".*");
            match regex::Regex::new(&format!("^{}$", pattern)) {
                Ok(regex) => regex.is_match(module_path),
                Err(_) => {
                    eprintln!("Invalid regex pattern: {}", pattern);
                    false
                }
            }
        } else {
            // Simple string matching
            module_path.contains(pattern)
        }
    }

    pub fn to_string_colored(&self) -> String {
        let level_str = self.to_string();
        match self {
            LogLevel::Error => level_str.red().to_string(),
            LogLevel::Warning => level_str.yellow().to_string(),
            LogLevel::Info => level_str.green().to_string(),
            LogLevel::Debug => level_str.blue().to_string(),
            LogLevel::Trace => level_str.cyan().to_string(),
            LogLevel::Success => level_str.green().to_string(),
            LogLevel::Critical => level_str.red().to_string(),
        }
    }
}

impl PartialOrd for LogLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.as_u8().cmp(&other.as_u8()))
    }
}

impl Ord for LogLevel {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_u8().cmp(&other.as_u8())
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warning => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Success => write!(f, "SUCCESS"),
            LogLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

impl FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "TRACE" => Ok(LogLevel::Trace),
            "DEBUG" => Ok(LogLevel::Debug),
            "INFO" => Ok(LogLevel::Info),
            "SUCCESS" => Ok(LogLevel::Success),
            "WARNING" => Ok(LogLevel::Warning),
            "ERROR" => Ok(LogLevel::Error),
            "CRITICAL" => Ok(LogLevel::Critical),
            _ => Err(format!("Invalid log level: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_ordering() {
        assert!(LogLevel::Trace < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Success);
        assert!(LogLevel::Success < LogLevel::Warning);
        assert!(LogLevel::Warning < LogLevel::Error);
        assert!(LogLevel::Error < LogLevel::Critical);
    }

    #[test]
    fn test_level_string_representation() {
        assert_eq!(LogLevel::Trace.as_str(), "TRACE");
        assert_eq!(LogLevel::Debug.as_str(), "DEBUG");
        assert_eq!(LogLevel::Info.as_str(), "INFO");
        assert_eq!(LogLevel::Success.as_str(), "SUCCESS");
        assert_eq!(LogLevel::Warning.as_str(), "WARNING");
        assert_eq!(LogLevel::Error.as_str(), "ERROR");
        assert_eq!(LogLevel::Critical.as_str(), "CRITICAL");
    }

    #[test]
    fn test_level_display() {
        assert_eq!(format!("{}", LogLevel::Info), "INFO");
        assert_eq!(format!("{}", LogLevel::Error), "ERROR");
    }

    #[test]
    fn test_level_numeric_values() {
        assert_eq!(LogLevel::Trace.as_u8(), 5);
        assert_eq!(LogLevel::Debug.as_u8(), 10);
        assert_eq!(LogLevel::Info.as_u8(), 20);
        assert_eq!(LogLevel::Success.as_u8(), 25);
        assert_eq!(LogLevel::Warning.as_u8(), 30);
        assert_eq!(LogLevel::Error.as_u8(), 40);
        assert_eq!(LogLevel::Critical.as_u8(), 50);
    }

    #[test]
    fn test_level_colors() {
        assert_eq!(LogLevel::Trace.color(), "\x1b[37m");
        assert_eq!(LogLevel::Debug.color(), "\x1b[34m");
        assert_eq!(LogLevel::Info.color(), "\x1b[32m");
        assert_eq!(LogLevel::Success.color(), "\x1b[36m");
        assert_eq!(LogLevel::Warning.color(), "\x1b[33m");
        assert_eq!(LogLevel::Error.color(), "\x1b[31m");
        assert_eq!(LogLevel::Critical.color(), "\x1b[35m");
    }

    #[test]
    fn test_level_from_str() {
        assert_eq!("trace".parse::<LogLevel>(), Ok(LogLevel::Trace));
        assert_eq!("DEBUG".parse::<LogLevel>(), Ok(LogLevel::Debug));
        assert_eq!("info".parse::<LogLevel>(), Ok(LogLevel::Info));
        assert_eq!("SUCCESS".parse::<LogLevel>(), Ok(LogLevel::Success));
        assert_eq!("warning".parse::<LogLevel>(), Ok(LogLevel::Warning));
        assert_eq!("ERROR".parse::<LogLevel>(), Ok(LogLevel::Error));
        assert_eq!("critical".parse::<LogLevel>(), Ok(LogLevel::Critical));
        assert!("invalid".parse::<LogLevel>().is_err());
    }

    #[test]
    fn test_level_from_str_edge_cases() {
        // Test mixed case
        assert_eq!("TrAcE".parse::<LogLevel>(), Ok(LogLevel::Trace));
        assert_eq!("DeBuG".parse::<LogLevel>(), Ok(LogLevel::Debug));

        // Test with whitespace
        assert!(" trace ".parse::<LogLevel>().is_err());
        assert!("debug ".parse::<LogLevel>().is_err());

        // Test empty string
        assert!("".parse::<LogLevel>().is_err());
    }

    #[test]
    fn test_level_comparisons() {
        // Test equality
        assert_eq!(LogLevel::Info, LogLevel::Info);
        assert_ne!(LogLevel::Info, LogLevel::Error);

        // Test ordering
        assert!(LogLevel::Trace < LogLevel::Critical);
        assert!(LogLevel::Critical > LogLevel::Trace);

        // Test PartialOrd implementation
        assert!(LogLevel::Trace <= LogLevel::Debug);
        assert!(LogLevel::Debug >= LogLevel::Trace);
        assert!(LogLevel::Info <= LogLevel::Info);
        assert!(LogLevel::Info >= LogLevel::Info);
    }

    #[test]
    fn test_color_code_format() {
        // Test that all color codes start with \x1b[
        assert!(LogLevel::Trace.color().starts_with("\x1b["));
        assert!(LogLevel::Debug.color().starts_with("\x1b["));
        assert!(LogLevel::Info.color().starts_with("\x1b["));
        assert!(LogLevel::Success.color().starts_with("\x1b["));
        assert!(LogLevel::Warning.color().starts_with("\x1b["));
        assert!(LogLevel::Error.color().starts_with("\x1b["));
        assert!(LogLevel::Critical.color().starts_with("\x1b["));

        // Test reset color format
        assert_eq!(LogLevel::reset_color(), "\x1b[0m");
    }

    #[test]
    fn test_level_emoji() {
        assert_eq!(LogLevel::Trace.emoji(), "🔍");
        assert_eq!(LogLevel::Debug.emoji(), "🐛");
        assert_eq!(LogLevel::Info.emoji(), "ℹ️");
        assert_eq!(LogLevel::Success.emoji(), "✅");
        assert_eq!(LogLevel::Warning.emoji(), "⚠️");
        assert_eq!(LogLevel::Error.emoji(), "❌");
        assert_eq!(LogLevel::Critical.emoji(), "💥");
    }

    #[test]
    fn test_module_pattern_matching() {
        let level = LogLevel::Info;

        // Simple string matching
        assert!(level.matches_module_pattern("my_app::module", "my_app"));
        assert!(level.matches_module_pattern("my_app::module", "module"));
        assert!(!level.matches_module_pattern("my_app::module", "other"));

        // Glob pattern matching
        assert!(level.matches_module_pattern("my_app::module", "my_app::*"));
        assert!(level.matches_module_pattern("my_app::module", "*::module"));
        assert!(!level.matches_module_pattern("my_app::module", "other::*"));
    }

    #[test]
    fn test_serialization() {
        let level = LogLevel::Info;
        let serialized = serde_json::to_string(&level).unwrap();
        assert_eq!(serialized, "\"Info\"");

        let deserialized: LogLevel = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, level);
    }
}
