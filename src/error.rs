//! Enhanced error types with helpful diagnostics
//!
//! This module provides detailed error messages with:
//! - Source location tracking
//! - Syntax highlighting of error locations
//! - Helpful suggestions for fixes
//! - Context-aware error messages

use std::fmt::Write;

/// Result type alias for nginx-discovery operations
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type with enhanced diagnostics
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error during parsing with detailed context
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
        /// Helpful suggestion to fix the error
        help: Option<String>,

    },

    /// Unexpected end of input while parsing
    #[error("Unexpected end of input")]
    UnexpectedEof {
        /// What was expected
        expected: String,
        /// Location where EOF occurred
        line: usize,
    },

    /// Invalid directive name or usage
    #[error("Invalid directive: {name}")]
    InvalidDirective {
        /// Directive name
        name: String,
        /// Reason why it's invalid
        reason: Option<String>,
        /// Suggestion for valid alternative
        suggestion: Option<String>,
    },

    /// Invalid argument for a directive
    #[error("Invalid argument for directive '{directive}': {message}")]
    InvalidArgument {
        /// Directive name
        directive: String,
        /// Error message
        message: String,
        /// Expected format
        expected: Option<String>,
    },

    /// Syntax error with context
    #[error("Syntax error: {message}")]
    Syntax {
        /// Error message
        message: String,
        /// Location
        line: usize,
        /// Column
        col: usize,
        /// What was expected
        expected: Option<String>,
        /// What was found
        found: Option<String>,
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

        /// Network-related errors
    #[error("Network error: {0}")]
    Network(String),

    /// Invalid input provided
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Feature not yet implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// Required feature not enabled
    #[error("Feature '{0}' not enabled. Enable it in Cargo.toml")]
    FeatureNotEnabled(String),
}

#[cfg(feature = "export-toml")]
impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Self {
        Self::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            err.to_string(),
        ))
    }
}

impl From<std::fmt::Error> for Error {
    fn from(err: std::fmt::Error) -> Self {
        Self::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            err.to_string(),
        ))
    }
}

impl Error {
    /// Create a new parse error with helpful context
    #[must_use]
    pub fn parse(message: impl Into<String>, line: usize, col: usize) -> Self {
        Self::Parse {
            message: message.into(),
            line,
            col,
            snippet: None,
            help: None,
        }
    }

    /// Create a parse error with source snippet and help text
    #[must_use]
    pub fn parse_with_context(
        message: impl Into<String>,
        line: usize,
        col: usize,
        snippet: impl Into<String>,
        help: impl Into<String>,
    ) -> Self {
        Self::Parse {
            message: message.into(),
            line,
            col,
            snippet: Some(snippet.into()),
            help: Some(help.into()),
        }
    }

    /// Create an unexpected EOF error
    #[must_use]
    pub fn unexpected_eof(expected: impl Into<String>, line: usize) -> Self {
        Self::UnexpectedEof {
            expected: expected.into(),
            line,
        }
    }

    /// Create a syntax error with expected/found information
    #[must_use]
    pub fn syntax(
        message: impl Into<String>,
        line: usize,
        col: usize,
        expected: Option<String>,
        found: Option<String>,
    ) -> Self {
        Self::Syntax {
            message: message.into(),
            line,
            col,
            expected,
            found,
        }
    }

    /// Create an invalid directive error with suggestion
    #[must_use]
    pub fn invalid_directive(
        name: impl Into<String>,
        reason: Option<String>,
        suggestion: Option<String>,
    ) -> Self {
        Self::InvalidDirective {
            name: name.into(),
            reason,
            suggestion,
        }
    }

    /// Create a custom error
    #[must_use]
    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom(message.into())
    }

    /// Get the error message for display
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::Parse { message, .. }
            | Self::InvalidArgument { message, .. }
            | Self::Syntax { message, .. } => message.clone(),
            Self::InvalidDirective { name, .. } => name.clone(),
            _ => self.to_string(),
        }
    }

    /// Get detailed error information with formatted output
    ///
    /// This provides a beautiful, colorized error display with:
    /// - Error location and message
    /// - Source code snippet
    /// - Visual pointer to the error
    /// - Helpful suggestions
    #[must_use]
    pub fn detailed(&self) -> String {
        match self {
            Self::Parse {
                message,
                line,
                col,
                snippet,
                help,
            } => format_parse_error(*line, *col, message, snippet.as_deref(), help.as_deref()),
            Self::Syntax {
                message,
                line,
                col,
                expected,
                found,
            } => format_syntax_error(*line, *col, message, expected.as_deref(), found.as_deref()),
            Self::UnexpectedEof { expected, line } => {
                format!("Unexpected end of file at line {line}\nExpected: {expected}")
            }
            Self::InvalidDirective {
                name,
                reason,
                suggestion,
            } => {
                let mut output = format!("Invalid directive: {name}");
                if let Some(r) = reason {
                    let _ = write!(output, "\nReason: {r}");
                }
                if let Some(s) = suggestion {
                    let _ = write!(output, "\nSuggestion: Try using '{s}' instead");
                }
                output
            }
            _ => self.to_string(),
        }
    }

    /// Get a short, one-line description
    #[must_use]
    pub fn short(&self) -> String {
        match self {
            Self::Parse {
                message, line, col, ..
            }
            | Self::Syntax {
                message, line, col, ..
            } => {
                format!("line {line}:{col}: {message}")
            }
            _ => self.to_string(),
        }
    }
}

/// Format a parse error with beautiful output
fn format_parse_error(
    line: usize,
    col: usize,
    message: &str,
    snippet: Option<&str>,
    help: Option<&str>,
) -> String {
    let mut output = format!("Parse error at line {line}, column {col}: {message}");

    if let Some(snippet) = snippet {
        let _ = writeln!(output, "\n");
        let _ = writeln!(output, "{snippet}");
        // Add pointer to error location
        let pointer = format!("{}^", " ".repeat(col.saturating_sub(1)));
        let _ = writeln!(output, "{pointer}");
    }

    if let Some(help) = help {
        let _ = writeln!(output, "\nHelp: {help}");
    }

    output
}

/// Format a syntax error with expected/found information
fn format_syntax_error(
    line: usize,
    col: usize,
    message: &str,
    expected: Option<&str>,
    found: Option<&str>,
) -> String {
    let mut output = format!("Syntax error at line {line}, column {col}: {message}");

    if let Some(exp) = expected {
        let _ = write!(output, "\nExpected: {exp}");
    }

    if let Some(fnd) = found {
        let _ = write!(output, "\nFound: {fnd}");
    }

    output
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
        assert_eq!(err.short(), "line 10:5: unexpected token");
    }

    #[test]
    fn test_parse_error_with_context() {
        let err = Error::parse_with_context(
            "unexpected semicolon",
            2,
            10,
            "server { listen 80;; }",
            "Remove the extra semicolon",
        );
        let detailed = err.detailed();
        assert!(detailed.contains("line 2"));
        assert!(detailed.contains("server { listen 80;; }"));
        assert!(detailed.contains('^'));
        assert!(detailed.contains("Help: Remove the extra semicolon"));
    }

    #[test]
    fn test_syntax_error() {
        let err = Error::syntax(
            "invalid token",
            5,
            12,
            Some("';' or '{'".to_string()),
            Some("'@'".to_string()),
        );
        let detailed = err.detailed();
        assert!(detailed.contains("Syntax error"));
        assert!(detailed.contains("Expected: ';' or '{'"));
        assert!(detailed.contains("Found: '@'"));
    }

    #[test]
    fn test_unexpected_eof() {
        let err = Error::unexpected_eof("closing brace '}'", 100);
        assert!(err.to_string().contains("Unexpected end of input"));
        let detailed = err.detailed();
        assert!(detailed.contains("line 100"));
        assert!(detailed.contains("Expected: closing brace '}'"));
    }

    #[test]
    fn test_invalid_directive() {
        let err = Error::invalid_directive(
            "liste",
            Some("Unknown directive".to_string()),
            Some("listen".to_string()),
        );
        let detailed = err.detailed();
        assert!(detailed.contains("Invalid directive: liste"));
        assert!(detailed.contains("Reason: Unknown directive"));
        assert!(detailed.contains("Try using 'listen' instead"));
    }

    #[test]
    fn test_custom_error() {
        let err = Error::custom("something went wrong");
        assert_eq!(err.message(), "something went wrong");
    }

    #[test]
    fn test_error_formatting() {
        let err = Error::parse_with_context(
            "missing semicolon",
            3,
            20,
            "server { listen 80 }",
            "Add a semicolon after '80'",
        );

        let detailed = err.detailed();
        // Should contain line number
        assert!(detailed.contains("line 3"));
        // Should contain snippet
        assert!(detailed.contains("server { listen 80 }"));
        // Should contain pointer
        assert!(detailed.contains('^'));
        // Should contain help
        assert!(detailed.contains("Help:"));
    }
}
