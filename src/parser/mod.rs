//! NGINX configuration parser

mod lexer;
mod token;

pub use lexer::Lexer;
pub use token::{Token, TokenKind};

use crate::ast::Config;
use crate::error::Result;

/// Parse NGINX configuration from text
pub fn parse(_input: &str) -> Result<Config> {
    // TODO: Implement full parser in Day 4
    Ok(Config::new())
}
