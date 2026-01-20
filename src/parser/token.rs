//! Token types for NGINX configuration lexer

use crate::ast::Span;

/// A token in the NGINX configuration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// The token type and value
    pub kind: TokenKind,
    /// Source location
    pub span: Span,
}

impl Token {
    /// Create a new token
    #[must_use]
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// Token types in NGINX configuration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    /// Word/identifier: `server`, `listen`, etc.
    Word(String),

    /// String literal: `"value"` or `'value'`
    String(String),

    /// Number: `80`, `443`, etc.
    Number(String),

    /// Variable: `$host`, `$remote_addr`
    Variable(String),

    /// Left brace: `{`
    LeftBrace,

    /// Right brace: `}`
    RightBrace,

    /// Semicolon: `;`
    Semicolon,

    /// Comment: `# comment text`
    Comment(String),

    /// End of file
    Eof,
}

impl TokenKind {
    /// Check if this token is a word
    #[must_use]
    pub fn is_word(&self) -> bool {
        matches!(self, Self::Word(_))
    }

    /// Check if this token is a string
    #[must_use]
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    /// Check if this token is a number
    #[must_use]
    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    /// Check if this token is a variable
    #[must_use]
    pub fn is_variable(&self) -> bool {
        matches!(self, Self::Variable(_))
    }

    /// Get the string value if this is a word or string token
    #[must_use]
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::Word(s) | Self::String(s) | Self::Variable(s) | Self::Number(s) => Some(s),
            _ => None,
        }
    }
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Word(s) => write!(f, "word '{s}'"),
            Self::String(s) => write!(f, "string \"{s}\""),
            Self::Number(s) => write!(f, "number '{s}'"),
            Self::Variable(s) => write!(f, "variable '${s}'"),
            Self::LeftBrace => write!(f, "'{{'"), // Changed: double {{ to escape
            Self::RightBrace => write!(f, "'}}'"), // Changed: double }} to escape
            Self::Semicolon => write!(f, "';'"),
            Self::Comment(s) => write!(f, "comment '# {s}'"),
            Self::Eof => write!(f, "end of file"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        let token = Token::new(TokenKind::Word("server".to_string()), Span::new(0, 6, 1, 1));
        assert_eq!(token.kind, TokenKind::Word("server".to_string()));
    }

    #[test]
    fn test_token_kind_checks() {
        assert!(TokenKind::Word("test".to_string()).is_word());
        assert!(TokenKind::String("test".to_string()).is_string());
        assert!(TokenKind::Number("80".to_string()).is_number());
        assert!(TokenKind::Variable("host".to_string()).is_variable());

        assert!(!TokenKind::LeftBrace.is_word());
        assert!(!TokenKind::Semicolon.is_string());
    }

    #[test]
    fn test_as_string() {
        assert_eq!(
            TokenKind::Word("test".to_string()).as_string(),
            Some("test")
        );
        assert_eq!(
            TokenKind::String("value".to_string()).as_string(),
            Some("value")
        );
        assert_eq!(TokenKind::LeftBrace.as_string(), None);
    }

    #[test]
    fn test_token_display() {
        assert_eq!(
            TokenKind::Word("server".to_string()).to_string(),
            "word 'server'"
        );
        assert_eq!(TokenKind::LeftBrace.to_string(), "'{'");
        assert_eq!(TokenKind::Semicolon.to_string(), "';'");
    }
}
