//! High-level extractors for NGINX directives

pub mod logs;
pub mod servers;

pub use logs::{access_logs, log_formats};
pub use servers::servers;
