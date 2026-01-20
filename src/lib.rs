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
//! ```rust,ignore
//! use nginx_discovery::prelude::*;
//!
//! // Discover from running NGINX instance
//! let discovery = NginxDiscovery::from_running_instance()?;
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
//! ```rust,ignore
//! use nginx_discovery::prelude::*;
//!
//! let discovery = NginxDiscovery::from_config_file("/etc/nginx/nginx.conf")?;
//! let logs = discovery.access_logs();
//!
//! for log in logs {
//!     println!("Path: {}", log.path.display());
//!     if let Some(format) = &log.format {
//!         println!("Format: {}", format.pattern);
//!     }
//! }
//! ```
//!
//! ### Mid-level API
//!
//! ```rust,ignore
//! use nginx_discovery::{parse, extract};
//!
//! let config_text = std::fs::read_to_string("/etc/nginx/nginx.conf")?;
//! let config = parse(&config_text)?;
//!
//! let logs = extract::access_logs(&config)?;
//! let servers = extract::servers(&config)?;
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![cfg_attr(docsrs, feature(doc_cfg))]

// Public modules
pub mod ast;
pub mod error;
pub mod extract;
pub mod parser;

#[cfg(feature = "system")]
#[cfg_attr(docsrs, doc(cfg(feature = "system")))]
pub mod system;

pub mod types;

#[cfg(feature = "visitor")]
#[cfg_attr(docsrs, doc(cfg(feature = "visitor")))]
pub mod visitor;

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
    // pub use crate::types::*;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parse() {
        let config = r#"
            log_format combined '$remote_addr - $remote_user [$time_local]';
            access_log /var/log/nginx/access.log combined;
        "#;

        let result = parse(config);
        assert!(result.is_ok());
    }
}
