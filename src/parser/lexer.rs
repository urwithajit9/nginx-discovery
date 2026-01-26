//! Lexer for NGINX configuration files
use crate::ast::Span;
use crate::error::{Error, Result};
use crate::parser::{Token, TokenKind};

/// Lexer for tokenizing NGINX configuration
pub struct Lexer<'a> {
    /// The input source code
    input: &'a str,
    /// Current position in bytes
    pos: usize,
    /// Current line number (1-indexed)
    line: usize,
    /// Current column number (1-indexed)
    col: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer
    #[must_use]
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    /// Get the next token
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - An unexpected character is encountered
    /// - A string literal is unterminated
    /// - A variable reference is malformed
    pub fn next_token(&mut self) -> Result<Token> {
        // Skip whitespace
        self.skip_whitespace();

        // Check for EOF
        if self.is_eof() {
            return Ok(self.make_token(TokenKind::Eof));
        }

        let start_pos = self.pos;
        let start_line = self.line;
        let start_col = self.col;

        let ch = self.current_char();

        let kind = match ch {
            // Comments
            '#' => self.lex_comment(),

            // Braces
            '{' => {
                self.advance();
                TokenKind::LeftBrace
            }
            '}' => {
                self.advance();
                TokenKind::RightBrace
            }

            // Semicolon
            ';' => {
                self.advance();
                TokenKind::Semicolon
            }

            // Equals (for options like buffer=32k)
            '=' => {
                self.advance();
                TokenKind::Word("=".to_string()) // Treat = as a word token
            }

            // Strings
            '"' => self.lex_string('"')?,
            '\'' => self.lex_string('\'')?,

            // Variables
            '$' => self.lex_variable()?,

            // Numbers or words
            _ if ch.is_ascii_digit() => self.lex_number(),
            _ if is_word_start(ch) => self.lex_word(),

            _ => {
                return Err(Error::syntax(
                    format!("unexpected character '{ch}'"),
                    self.line,
                    self.col,
                    Some("valid token".to_string()),
                    Some(format!("'{ch}'")),
                ));
            }
        };

        let span = Span::new(start_pos, self.pos, start_line, start_col);
        Ok(Token::new(kind, span))
    }

    /// Tokenize the entire input
    ///
    /// # Errors
    ///
    /// Returns an error if any token cannot be parsed.
    /// See [`next_token`](Self::next_token) for specific error conditions.
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);

            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }

    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while !self.is_eof() {
            let ch = self.current_char();
            if ch.is_whitespace() {
                if ch == '\n' {
                    self.line += 1;
                    self.col = 1;
                    self.pos += 1;
                } else {
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    /// Lex a comment
    fn lex_comment(&mut self) -> TokenKind {
        self.advance(); // Skip '#'

        let start = self.pos;
        while !self.is_eof() && self.current_char() != '\n' {
            self.advance();
        }

        let comment = self.input[start..self.pos].trim().to_string();
        TokenKind::Comment(comment)
    }

    /// Lex a quoted string
    fn lex_string(&mut self, quote: char) -> Result<TokenKind> {
        self.advance(); // Skip opening quote

        let start = self.pos;
        let mut escaped = false;

        while !self.is_eof() {
            let ch = self.current_char();

            if escaped {
                escaped = false;
                self.advance();
                continue;
            }

            if ch == '\\' {
                escaped = true;
                self.advance();
                continue;
            }

            if ch == quote {
                let value = self.input[start..self.pos].to_string();
                self.advance(); // Skip closing quote
                return Ok(TokenKind::String(value));
            }

            if ch == '\n' {
                return Err(Error::syntax(
                    "unterminated string literal",
                    self.line,
                    self.col,
                    Some("closing quote".to_string()),
                    Some("newline".to_string()),
                ));
            }

            self.advance();
        }

        Err(Error::unexpected_eof("closing quote", self.line))
    }

    /// Lex a variable ($name)
    fn lex_variable(&mut self) -> Result<TokenKind> {
        self.advance(); // Skip '$'

        let start = self.pos;

        // Variable name can be in braces: ${var_name}
        if !self.is_eof() && self.current_char() == '{' {
            self.advance(); // Skip '{'
            let name_start = self.pos;

            while !self.is_eof() && self.current_char() != '}' {
                self.advance();
            }

            if self.is_eof() {
                return Err(Error::unexpected_eof("'}'", self.line));
            }

            let name = self.input[name_start..self.pos].to_string();
            self.advance(); // Skip '}'
            return Ok(TokenKind::Variable(name));
        }

        // Regular variable: $name
        while !self.is_eof() && is_word_char(self.current_char()) {
            self.advance();
        }

        let name = self.input[start..self.pos].to_string();

        if name.is_empty() {
            return Err(Error::syntax(
                "expected variable name after '$'",
                self.line,
                self.col,
                Some("variable name".to_string()),
                None,
            ));
        }

        Ok(TokenKind::Variable(name))
    }

    /// Lex a number
    fn lex_number(&mut self) -> TokenKind {
        let start = self.pos;

        while !self.is_eof() && (self.current_char().is_ascii_digit() || self.current_char() == '.')
        {
            self.advance();
        }

        let number = self.input[start..self.pos].to_string();
        TokenKind::Number(number)
    }

    /// Lex a word (identifier)
    fn lex_word(&mut self) -> TokenKind {
        let start = self.pos;

        while !self.is_eof() && is_word_char(self.current_char()) {
            self.advance();
        }

        let word = self.input[start..self.pos].to_string();
        TokenKind::Word(word)
    }

    /// Make a token at current position
    fn make_token(&self, kind: TokenKind) -> Token {
        Token::new(kind, Span::new(self.pos, self.pos, self.line, self.col))
    }

    /// Get current character
    fn current_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap_or('\0')
    }

    /// Check if at end of file
    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Advance to next character
    fn advance(&mut self) {
        if !self.is_eof() {
            let ch = self.current_char();
            self.pos += ch.len_utf8();
            if ch != '\n' {
                self.col += 1;
            }
        }
    }
}

/// Check if character can start a word
fn is_word_start(ch: char) -> bool {
    ch.is_ascii_alphabetic()
        || ch == '_'
        || ch == '/'
        || ch == '.'
        || ch == '*'
        || ch == '^'
        || ch == '~'
        || ch == '\\'
}

/// Check if character can be part of a word
fn is_word_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric()
        || ch == '_'
        || ch == '-'
        || ch == '/'
        || ch == '.'
        || ch == ':'
        || ch == '='
        || ch == '*'
        || ch == '^'
        || ch == '~'
        || ch == '\\'
        || ch == '$' // Add $ too for regex patterns like $
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_simple_directive() {
        let mut lexer = Lexer::new("user nginx;");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 4); // user, nginx, ;, EOF
        assert_eq!(tokens[0].kind, TokenKind::Word("user".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::Word("nginx".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Semicolon);
        assert_eq!(tokens[3].kind, TokenKind::Eof);
    }

    #[test]
    fn test_lex_block() {
        let mut lexer = Lexer::new("server { listen 80; }");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Word("server".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::LeftBrace);
        assert_eq!(tokens[2].kind, TokenKind::Word("listen".to_string()));
        assert_eq!(tokens[3].kind, TokenKind::Number("80".to_string()));
        assert_eq!(tokens[4].kind, TokenKind::Semicolon);
        assert_eq!(tokens[5].kind, TokenKind::RightBrace);
    }

    #[test]
    fn test_lex_string() {
        let mut lexer = Lexer::new(r#"root "/var/www";"#);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Word("root".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::String("/var/www".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Semicolon);
    }

    #[test]
    fn test_lex_variable() {
        let mut lexer = Lexer::new("set $host;");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Word("set".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::Variable("host".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Semicolon);
    }

    #[test]
    fn test_lex_comment() {
        let mut lexer = Lexer::new("# This is a comment\nuser nginx;");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(
            tokens[0].kind,
            TokenKind::Comment("This is a comment".to_string())
        );
        assert_eq!(tokens[1].kind, TokenKind::Word("user".to_string()));
    }

    #[test]
    fn test_position_tracking() {
        let mut lexer = Lexer::new("server\n{\n  listen 80;\n}");
        let tokens = lexer.tokenize().unwrap();

        // Check that positions are tracked
        assert_eq!(tokens[0].span.line, 1);
        assert_eq!(tokens[1].span.line, 2);
        assert_eq!(tokens[2].span.line, 3);
    }

    #[test]
    fn test_unterminated_string() {
        let mut lexer = Lexer::new(r#"root "/var/www"#);
        let result = lexer.tokenize();

        assert!(result.is_err());
    }
}
