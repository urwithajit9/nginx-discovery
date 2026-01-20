//! NGINX configuration parser

mod lexer;
mod parse;
mod token;

pub use lexer::Lexer;
pub use parse::Parser;
pub use token::{Token, TokenKind};

use crate::ast::Config;
use crate::error::Result;

/// Parse NGINX configuration from text
///
/// This is the main entry point for parsing.
///
/// # Examples
///
/// ```
/// use nginx_discovery::parse;
///
/// let config = "user nginx;";
/// let result = parse(config).unwrap();
/// assert_eq!(result.directives.len(), 1);
/// ```
pub fn parse(input: &str) -> Result<Config> {
    let mut parser = Parser::new(input)?;
    parser.parse()
}
