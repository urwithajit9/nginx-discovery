//! High-level extractors for NGINX directives

pub mod logs;

pub use logs::{access_logs, log_formats};
