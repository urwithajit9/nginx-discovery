//! Error builder for constructing detailed error messages
//!
//! Provides a fluent API for building errors with context.

use crate::error::Error;

/// Builder for constructing parse errors with context
#[derive(Debug, Default)]
pub struct ErrorBuilder {
    message: String,
    line: usize,
    col: usize,
    snippet: Option<String>,
    help: Option<String>,
}

impl ErrorBuilder {
    /// Create a new error builder
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the error message
    #[must_use]
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    /// Set the location (line and column)
    #[must_use]
    pub fn location(mut self, line: usize, col: usize) -> Self {
        self.line = line;
        self.col = col;
        self
    }

    /// Set the source code snippet
    #[must_use]
    pub fn snippet(mut self, snippet: impl Into<String>) -> Self {
        self.snippet = Some(snippet.into());
        self
    }

    /// Set helpful suggestion
    #[must_use]
    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Build the error
    #[must_use]
    pub fn build(self) -> Error {
        if let (Some(snippet), Some(help)) = (self.snippet, self.help) {
            Error::parse_with_context(self.message, self.line, self.col, snippet, help)
        } else {
            Error::parse(self.message, self.line, self.col)
        }
    }
}

/// Extract a snippet from source text around a specific line
///
/// # Arguments
/// * `source` - The full source text
/// * `line` - The line number (1-indexed)
/// * `context_lines` - Number of lines to show before and after
///
/// # Returns
/// A string containing the relevant lines
#[must_use]
pub fn extract_snippet(source: &str, line: usize, context_lines: usize) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let line_idx = line.saturating_sub(1);

    let start = line_idx.saturating_sub(context_lines);
    let end = (line_idx + context_lines + 1).min(lines.len());

    lines[start..end].join("\n")
}

/// Get the line at a specific index
pub fn get_line(source: &str, line: usize) -> Option<String> {
    source.lines().nth(line.saturating_sub(1)).map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_builder_basic() {
        let error = ErrorBuilder::new()
            .message("unexpected token")
            .location(5, 10)
            .build();

        assert!(error.to_string().contains("line 5"));
        assert!(error.to_string().contains("column 10"));
    }

    #[test]
    fn test_error_builder_with_context() {
        let error = ErrorBuilder::new()
            .message("missing semicolon")
            .location(10, 20)
            .snippet("server { listen 80 }")
            .help("Add a semicolon after '80'")
            .build();

        let detailed = error.detailed();
        assert!(detailed.contains("missing semicolon"));
        assert!(detailed.contains("server { listen 80 }"));
        assert!(detailed.contains("Help: Add a semicolon"));
    }

    #[test]
    fn test_extract_snippet() {
        let source = "line 1\nline 2\nline 3\nline 4\nline 5";

        let snippet = extract_snippet(source, 3, 1);
        assert_eq!(snippet, "line 2\nline 3\nline 4");

        let snippet = extract_snippet(source, 1, 1);
        assert_eq!(snippet, "line 1\nline 2");

        let snippet = extract_snippet(source, 5, 1);
        assert_eq!(snippet, "line 4\nline 5");
    }

    #[test]
    fn test_get_line() {
        let source = "line 1\nline 2\nline 3";

        assert_eq!(get_line(source, 1), Some("line 1".to_string()));
        assert_eq!(get_line(source, 2), Some("line 2".to_string()));
        assert_eq!(get_line(source, 3), Some("line 3".to_string()));
        assert_eq!(get_line(source, 4), None);
    }

    #[test]
    fn test_builder_fluent_api() {
        let error = ErrorBuilder::new()
            .message("test error")
            .location(1, 1)
            .snippet("test snippet")
            .help("test help")
            .build();

        assert!(error.detailed().contains("test error"));
        assert!(error.detailed().contains("test snippet"));
        assert!(error.detailed().contains("test help"));
    }
}
