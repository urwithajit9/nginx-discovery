//! Access log type

use std::collections::HashMap;
use std::path::PathBuf;

/// Represents an NGINX `access_log` directive
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AccessLog {
    /// Path to the log file
    pub path: PathBuf,

    /// Format name (e.g., "combined", "main")
    pub format_name: Option<String>,

    /// Additional options (buffer, gzip, etc.)
    pub options: HashMap<String, String>,

    /// Context where this log was defined
    pub context: LogContext,
}

/// Context where a log directive appears
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LogContext {
    /// Main/http context
    Main,
    /// Server block
    Server(String),
    /// Location block
    Location(String),
}

impl AccessLog {
    /// Create a new access log
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            format_name: None,
            options: HashMap::new(),
            context: LogContext::Main,
        }
    }

    /// Set the format name
    #[must_use]
    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format_name = Some(format.into());
        self
    }

    /// Set the context
    #[must_use]
    pub fn with_context(mut self, context: LogContext) -> Self {
        self.context = context;
        self
    }

    /// Add an option
    #[must_use]
    pub fn with_option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }
}
