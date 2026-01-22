//! Parser for NGINX configuration files
use crate::ast::{Config, Directive, Value};
use crate::error::{Error, Result};
// use crate::prelude::ErrorBuilder;
use crate::parser::{Lexer, Token, TokenKind};

/// Parser for NGINX configuration
pub struct Parser {
    /// Tokens to parse
    tokens: Vec<Token>,
    /// Current position in token stream
    pos: usize,
}

impl Parser {
    /// Create a new parser from source text
    ///
    /// # Errors
    ///
    /// Returns an error if tokenization fails.
    pub fn new(input: &str) -> Result<Self> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;

        Ok(Self { tokens, pos: 0 })
    }

    /// Parse the configuration
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Unexpected tokens are encountered
    /// - Directives are malformed
    /// - Blocks are not properly closed
    pub fn parse(&mut self) -> Result<Config> {
        let mut directives = Vec::new();

        while !self.is_eof() {
            // Skip comments
            if self.check_comment() {
                self.advance();
                continue;
            }

            let directive = self.parse_directive()?;
            directives.push(directive);
        }

        Ok(Config::with_directives(directives))
    }

    /// Parse a single directive (simple or block)
    fn parse_directive(&mut self) -> Result<Directive> {
        let _start_token = self.current();
        let name = self.expect_word()?;

        let mut args = Vec::new();

        // Collect arguments until we hit ; or {
        while !self.check(&TokenKind::Semicolon)
            && !self.check(&TokenKind::LeftBrace)
            && !self.is_eof()
        {
            if self.check_comment() {
                self.advance();
                continue;
            }

            let arg = self.parse_value()?;
            args.push(arg);
        }

        // Check if it's a block or simple directive
        if self.check(&TokenKind::LeftBrace) {
            // Block directive
            self.advance(); // consume {

            let children = self.parse_block_contents()?;

            self.expect(&TokenKind::RightBrace)?;

            Ok(Directive::block_with_values(name, args, children))
        } else {
            // Simple directive
            self.expect(&TokenKind::Semicolon)?;

            Ok(Directive::simple_with_values(name, args))
        }
    }

    /// Parse the contents of a block
    fn parse_block_contents(&mut self) -> Result<Vec<Directive>> {
        let mut directives = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_eof() {
            if self.check_comment() {
                self.advance();
                continue;
            }

            let directive = self.parse_directive()?;
            directives.push(directive);
        }

        Ok(directives)
    }

    /// Parse a value (string, number, word, variable)
    fn parse_value(&mut self) -> Result<Value> {
        let token = self.current();

        let value = match &token.kind {
            TokenKind::String(s) => Value::single_quoted(s.clone()),
            TokenKind::Word(s) | TokenKind::Number(s) => Value::literal(s.clone()),
            TokenKind::Variable(s) => Value::variable(s.clone()),
            _ => {
                return Err(Error::syntax(
                    "expected value",
                    token.span.line,
                    token.span.col,
                    Some("word, string, number, or variable".to_string()),
                    Some(format!("{}", token.kind)),
                ));
            }
        };

        self.advance();
        Ok(value)
    }

    /// Expect a specific token kind
    fn expect(&mut self, kind: &TokenKind) -> Result<Token> {
        let token = self.current().clone(); // Clone here

        if std::mem::discriminant(&token.kind) == std::mem::discriminant(kind) {
            self.advance();
            Ok(token) // No need to clone again
        } else {
            Err(Error::syntax(
                "unexpected token".to_string(),
                token.span.line,
                token.span.col,
                Some(format!("{kind}")),
                Some(format!("{}", token.kind)),
            ))
        }
    }

    /// Expect a word token and return its value
    fn expect_word(&mut self) -> Result<String> {
        let token = self.current();

        if let TokenKind::Word(name) = &token.kind {
            let result = name.clone();
            self.advance();
            Ok(result)
        } else {
            Err(Error::syntax(
                "expected directive name",
                token.span.line,
                token.span.col,
                Some("word".to_string()),
                Some(format!("{}", token.kind)),
            ))
        }
    }

    /// Get current token
    fn current(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .unwrap_or(&self.tokens[self.tokens.len() - 1])
    }

    /// Check if current token matches a kind
    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_eof() {
            return false;
        }
        std::mem::discriminant(&self.current().kind) == std::mem::discriminant(kind)
    }

    /// Check if current token is a comment
    fn check_comment(&self) -> bool {
        matches!(self.current().kind, TokenKind::Comment(_))
    }

    /// Check if at end of tokens
    fn is_eof(&self) -> bool {
        matches!(self.current().kind, TokenKind::Eof)
    }

    /// Advance to next token
    fn advance(&mut self) {
        if !self.is_eof() {
            self.pos += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_directive() {
        let input = "user nginx;";
        let mut parser = Parser::new(input).unwrap();
        let config = parser.parse().unwrap();

        assert_eq!(config.directives.len(), 1);
        assert_eq!(config.directives[0].name(), "user");
        assert_eq!(config.directives[0].args().len(), 1);
    }

    #[test]
    fn test_parse_multiple_directives() {
        let input = "user nginx;\nworker_processes auto;";
        let mut parser = Parser::new(input).unwrap();
        let config = parser.parse().unwrap();

        assert_eq!(config.directives.len(), 2);
        assert_eq!(config.directives[0].name(), "user");
        assert_eq!(config.directives[1].name(), "worker_processes");
    }

    #[test]
    fn test_parse_block_directive() {
        let input = "server { listen 80; }";
        let mut parser = Parser::new(input).unwrap();
        let config = parser.parse().unwrap();

        assert_eq!(config.directives.len(), 1);
        assert!(config.directives[0].is_block());
        assert_eq!(config.directives[0].children().unwrap().len(), 1);
    }

    #[test]
    fn test_parse_nested_blocks() {
        let input = r"
http {
    server {
        listen 80;
        location / {
            root /var/www;
        }
    }
}
";
        let mut parser = Parser::new(input).unwrap();
        let config = parser.parse().unwrap();

        assert_eq!(config.directives.len(), 1);

        let http = &config.directives[0];
        assert_eq!(http.name(), "http");
        assert_eq!(http.children().unwrap().len(), 1);

        let server = &http.children().unwrap()[0];
        assert_eq!(server.name(), "server");
        assert_eq!(server.children().unwrap().len(), 2); // listen + location
    }

    #[test]
    fn test_parse_with_comments() {
        let input = r"
# Main config
user nginx;  # Run as nginx
";
        let mut parser = Parser::new(input).unwrap();
        let config = parser.parse().unwrap();

        // Comments should be skipped
        assert_eq!(config.directives.len(), 1);
        assert_eq!(config.directives[0].name(), "user");
    }

    #[test]
    fn test_parse_strings() {
        let input = r#"root "/var/www/html";"#;
        let mut parser = Parser::new(input).unwrap();
        let config = parser.parse().unwrap();

        assert_eq!(config.directives.len(), 1);
        assert_eq!(config.directives[0].args().len(), 1);
    }

    #[test]
    fn test_parse_variables() {
        let input = "set $host localhost;";
        let mut parser = Parser::new(input).unwrap();
        let config = parser.parse().unwrap();

        assert_eq!(config.directives.len(), 1);
        assert!(config.directives[0].args()[0].is_variable());
    }
}
