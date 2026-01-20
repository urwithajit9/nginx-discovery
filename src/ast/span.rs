//! Source location tracking for error reporting

use std::fmt;

/// Represents a location in the source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Span {
    /// Starting byte offset in the source
    pub start: usize,
    /// Ending byte offset in the source
    pub end: usize,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub col: usize,
}

impl Span {
    /// Create a new span
    #[must_use]
    pub fn new(start: usize, end: usize, line: usize, col: usize) -> Self {
        Self {
            start,
            end,
            line,
            col,
        }
    }

    /// Create a span at a specific position with zero length
    #[must_use]
    pub fn at(pos: usize, line: usize, col: usize) -> Self {
        Self {
            start: pos,
            end: pos,
            line,
            col,
        }
    }

    /// Combine two spans into one that covers both
    #[must_use]
    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line: self.line.min(other.line),
            col: self.col.min(other.col),
        }
    }

    /// Get the length of the span in bytes
    #[must_use]
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Check if the span is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Get a substring from source text using this span
    #[must_use]
    pub fn slice<'a>(&self, source: &'a str) -> Option<&'a str> {
        source.get(self.start..self.end)
    }
}

impl Default for Span {
    fn default() -> Self {
        Self {
            start: 0,
            end: 0,
            line: 1,
            col: 1,
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.col)
    }
}

/// Trait for types that have a source location
pub trait Spanned {
    /// Get the span of this item
    fn span(&self) -> Span;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_new() {
        let span = Span::new(0, 10, 1, 1);
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 10);
        assert_eq!(span.line, 1);
        assert_eq!(span.col, 1);
    }

    #[test]
    fn test_span_at() {
        let span = Span::at(5, 2, 3);
        assert_eq!(span.start, 5);
        assert_eq!(span.end, 5);
        assert_eq!(span.line, 2);
        assert_eq!(span.col, 3);
        assert!(span.is_empty());
    }

    #[test]
    fn test_span_merge() {
        let span1 = Span::new(0, 5, 1, 1);
        let span2 = Span::new(10, 15, 2, 1);
        let merged = span1.merge(span2);

        assert_eq!(merged.start, 0);
        assert_eq!(merged.end, 15);
        assert_eq!(merged.line, 1);
    }

    #[test]
    fn test_span_len() {
        let span = Span::new(5, 15, 1, 1);
        assert_eq!(span.len(), 10);

        let empty_span = Span::at(5, 1, 1);
        assert_eq!(empty_span.len(), 0);
    }

    #[test]
    fn test_span_slice() {
        let source = "hello world";
        let span = Span::new(0, 5, 1, 1);
        assert_eq!(span.slice(source), Some("hello"));

        let span2 = Span::new(6, 11, 1, 7);
        assert_eq!(span2.slice(source), Some("world"));
    }

    #[test]
    fn test_span_display() {
        let span = Span::new(0, 5, 10, 25);
        assert_eq!(span.to_string(), "line 10, column 25");
    }

    #[test]
    fn test_span_default() {
        let span = Span::default();
        assert_eq!(span.line, 1);
        assert_eq!(span.col, 1);
        assert_eq!(span.start, 0);
    }
}
