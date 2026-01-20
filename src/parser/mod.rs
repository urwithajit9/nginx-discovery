//! NGINX configuration parser
//!
//! This module will contain the lexer and parser implementation.

use crate::ast::Config;
use crate::error::Result;

/// Parse NGINX configuration from text
///
/// This is a stub implementation. Full parser coming soon.
pub fn parse(_input: &str) -> Result<Config> {
    // TODO: Implement parser
    Ok(Config::new())
}
