//! # nginx-discovery
//!
//! Discover and introspect NGINX configurations with ease.
//!
//! This crate provides three levels of API for working with NGINX configurations:
//!
//! 1. **High-level Discovery API**: Simple methods for common use cases
//! 2. **Mid-level Extractors**: Type-safe directive extraction
//! 3. **Low-level Parser**: Direct AST access for custom processing
//!
//! ## Quick Start
//!
//! ```
//! use nginx_discovery::prelude::*;
//!
//! # fn main() -> nginx_discovery::Result<()> {
//! // Parse configuration text
//! let config = r"
//!     http {
//!         access_log /var/log/nginx/access.log;
//!     }
//! ";
//!
//! let discovery = NginxDiscovery::from_config_text(config)?;
//!
//! // Get access logs
//! for log in discovery.access_logs() {
//!     println!("Log: {}", log.path.display());
//! }
//!
//! // Get all server names
//! for name in discovery.server_names() {
//!     println!("Server: {}", name);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Feature Flags
//!
//! - `system` (default): System interaction (detect nginx, run nginx -T)
//! - `serde`: JSON/YAML serialization support
//! - `visitor`: Visitor pattern for AST traversal
//! - `includes`: Include directive resolution
//! - `cli`: Command-line interface (binary only)
//!
//! ## Examples
//!
//! ### Extract Log Files
//!
//! ```
//! use nginx_discovery::prelude::*;
//!
//! # fn main() -> nginx_discovery::Result<()> {
//! let config = r"
//!     access_log /var/log/nginx/access.log combined;
//!     log_format combined '$remote_addr $request';
//! ";
//!
//! let discovery = NginxDiscovery::from_config_text(config)?;
//! let logs = discovery.access_logs();
//!
//! for log in logs {
//!     println!("Path: {}", log.path.display());
//!     if let Some(format_name) = &log.format_name {
//!         println!("Format: {}", format_name);
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Parse from File
//!
//! ```no_run
//! use nginx_discovery::prelude::*;
//!
//! # fn main() -> nginx_discovery::Result<()> {
//! let discovery = NginxDiscovery::from_config_file("/etc/nginx/nginx.conf")?;
//! let logs = discovery.access_logs();
//! println!("Found {} access logs", logs.len());
//! # Ok(())
//! # }
//! ```
//!
//! ### Mid-level API
//!
//! ```
//! use nginx_discovery::{parse, extract};
//!
//! # fn main() -> nginx_discovery::Result<()> {
//! let config_text = "access_log /var/log/nginx/access.log;";
//! let config = parse(config_text)?;
//!
//! let logs = extract::access_logs(&config)?;
//! assert_eq!(logs.len(), 1);
//! # Ok(())
//! # }
//! ```
//!
//! ### Low-level Parser
//!
//! ```
//! use nginx_discovery::parse;
//!
//! # fn main() -> nginx_discovery::Result<()> {
//! let config = parse("user nginx;")?;
//! assert_eq!(config.directives.len(), 1);
//! assert_eq!(config.directives[0].name(), "user");
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![cfg_attr(docsrs, feature(doc_cfg))]

// Public modules
pub mod ast;
pub mod error;
pub mod error_builder;
pub mod extract;
pub mod parser;

#[cfg(feature = "system")]
#[cfg_attr(docsrs, doc(cfg(feature = "system")))]
pub mod system;

pub mod types;

#[cfg(feature = "visitor")]
#[cfg_attr(docsrs, doc(cfg(feature = "visitor")))]
pub mod visitor;

#[cfg(feature = "serde")]
pub mod export;

#[cfg(feature = "network")]
pub mod network;

// High-level API
mod discovery;
pub use discovery::NginxDiscovery;

// Re-exports for convenience
pub use error::{Error, Result};
pub use parser::parse;

/// Commonly used imports for quick setup
///
/// ```rust
/// use nginx_discovery::prelude::*;
/// ```
pub mod prelude {
    pub use crate::discovery::NginxDiscovery;
    pub use crate::error::{Error, Result};
    pub use crate::error_builder::ErrorBuilder;
    pub use crate::parser::{Lexer, Parser, Token, TokenKind};
    pub use crate::types::*;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parse() {
        let config = r"
            user nginx;
            worker_processes auto;
        ";

        let result = parse(config);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.directives.len(), 2);
        assert_eq!(config.directives[0].name(), "user");
        assert_eq!(config.directives[1].name(), "worker_processes");
    }

    #[test]
    fn test_discovery_from_text() {
        let config = "user nginx;";
        let discovery = NginxDiscovery::from_config_text(config);
        assert!(discovery.is_ok());
    }

    #[test]
    fn test_prelude_imports() {
        use crate::prelude::*;

        let config = "user nginx;";
        let discovery = NginxDiscovery::from_config_text(config).unwrap();
        assert_eq!(discovery.config().directives.len(), 1);
    }
}
