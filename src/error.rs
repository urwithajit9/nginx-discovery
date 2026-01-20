//! Error types for nginx-discovery

/// Result type alias for nginx-discovery operations
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for nginx-discovery
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error during parsing
    #[error("Parse error at line {line}, column {col}: {message}")]
    Parse {
        /// Error message
        message: String,
        /// Line number (1-indexed)
        line: usize,
        /// Column number (1-indexed)
        col: usize,
        /// Source snippet if available
        snippet: Option<String>,
    },

    /// Unexpected end of input
    #[error("Unexpected end of input")]
    UnexpectedEof,

    /// Invalid directive
    #[error("Invalid directive: {0}")]
    InvalidDirective(String),

    /// Invalid argument
    #[error("Invalid argument for directive '{directive}': {message}")]
    InvalidArgument {
        /// Directive name
        directive: String,
        /// Error message
        message: String,
    },

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// System error (nginx not found, etc.)
    #[cfg(feature = "system")]
    #[error("System error: {0}")]
    System(String),

    /// Serialization error
    #[cfg(feature = "serde")]
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Include resolution error
    #[cfg(feature = "includes")]
    #[error("Include resolution error: {0}")]
    Include(String),

    /// Custom error
    #[error("{0}")]
    Custom(String),
}

impl Error {
    /// Create a new parse error
    pub fn parse(message: impl Into<String>, line: usize, col: usize) -> Self {
        Self::Parse {
            message: message.into(),
            line,
            col,
            snippet: None,
        }
    }

    /// Create a parse error with source snippet
    pub fn parse_with_snippet(
        message: impl Into<String>,
        line: usize,
        col: usize,
        snippet: impl Into<String>,
    ) -> Self {
        Self::Parse {
            message: message.into(),
            line,
            col,
            snippet: Some(snippet.into()),
        }
    }

    /// Create a custom error
    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom(message.into())
    }

    /// Get the error message for display
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::Parse { message, .. } | Self::InvalidArgument { message, .. } => message.clone(),
            Self::InvalidDirective(msg) => msg.clone(),
            _ => self.to_string(),
        }
    }

    /// Get detailed error information with context
    #[must_use]
    pub fn detailed(&self) -> String {
        match self {
            Self::Parse {
                message,
                line,
                col,
                snippet,
            } => {
                use std::fmt::Write;
                let mut output = format!("Parse error at line {line}, column {col}: {message}");
                if let Some(snippet) = snippet {
                    let _ = writeln!(output, "\n\n{snippet}");
                    let _ = writeln!(output, "{}^", " ".repeat(*col - 1));
                }
                output
            }
            _ => self.to_string(),
        }
    }
}

#[cfg(feature = "serde")]
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

#[cfg(feature = "serde")]
impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error() {
        let err = Error::parse("unexpected token", 10, 5);
        assert!(err.to_string().contains("line 10"));
        assert!(err.to_string().contains("column 5"));
    }

    #[test]
    fn test_parse_error_with_snippet() {
        let err = Error::parse_with_snippet("unexpected semicolon", 2, 10, "server { listen 80;");
        let detailed = err.detailed();
        assert!(detailed.contains("line 2"));
        assert!(detailed.contains("server { listen 80;"));
        assert!(detailed.contains("^")); // Pointer to error
    }

    #[test]
    fn test_custom_error() {
        let err = Error::custom("something went wrong");
        assert_eq!(err.message(), "something went wrong");
    }
}
