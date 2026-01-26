// src/types/error_log.rs
use crate::types::LogContext;
use std::path::PathBuf;
use std::str::FromStr;

/// Represents an NGINX `error_log` directive
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ErrorLog {
    /// Path to the error log file
    pub path: PathBuf,
    /// Log level
    pub level: ErrorLogLevel,
    /// Context where this log was defined
    pub context: LogContext,
}

impl ErrorLog {
    /// Create a new error log
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            level: ErrorLogLevel::Error,
            context: LogContext::Main,
        }
    }

    /// Set the log level
    #[must_use]
    pub fn with_level(mut self, level: ErrorLogLevel) -> Self {
        self.level = level;
        self
    }

    /// Set the context
    #[must_use]
    pub fn with_context(mut self, context: LogContext) -> Self {
        self.context = context;
        self
    }
}

/// Error log levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ErrorLogLevel {
    /// Debug level
    Debug,
    /// Info level
    Info,
    /// Notice level
    Notice,
    /// Warn level
    Warn,
    /// Error level (default)
    Error,
    /// Crit level
    Crit,
    /// Alert level
    Alert,
    /// Emerg level
    Emerg,
}

impl FromStr for ErrorLogLevel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "debug" => Self::Debug,
            "info" => Self::Info,
            "notice" => Self::Notice,
            "warn" => Self::Warn,
            // "error" => Self::Error,  // same as default
            "crit" => Self::Crit,
            "alert" => Self::Alert,
            "emerg" => Self::Emerg,
            _ => Self::Error, // Default
        })
    }
}
