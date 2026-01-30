// src/network/mod.rs
//! Network health checking and validation.
//!
//! This module orchestrates all network-related checks (DNS, ports, SSL,
//! upstreams) across a parsed NGINX configuration.
//!
//! Individual checks live in submodules. This file is responsible for
//! *running them across the whole config* and normalizing results.
//!
//! # Examples
//!
//! ```no_run
//! use nginx_discovery::{parse, network::{check_all, NetworkCheckOptions}};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = parse("server { listen 80; server_name example.com; }")?;
//!     let options = NetworkCheckOptions::default();
//!
//!     let results = check_all(&config, options).await?;
//!     println!("Performed {} checks", results.len());
//!     Ok(())
//! }
//! ```

// -----------------------------------------------------------------------------
// Submodules
// -----------------------------------------------------------------------------

pub mod dns;
pub mod port;
pub mod ssl;
pub mod types;
pub mod upstream;

// -----------------------------------------------------------------------------
// Public re-exports (stable API)
// -----------------------------------------------------------------------------

pub use types::{
    CheckSeverity, DnsCheckResult, HealthCheckResult, HealthStatus, NetworkCheckOptions,
    PortCheckResult, SslCheckResult,
};

#[cfg(feature = "network")]
pub use dns::resolve_hostname;

#[cfg(feature = "network")]
pub use dns::resolve_hostnames;

#[cfg(feature = "network")]
pub use port::check_port;

#[cfg(feature = "network")]
pub use ssl::check_ssl_certificate;

#[cfg(feature = "network")]
pub use upstream::check_upstream_backend;

#[cfg(feature = "network")]
pub use upstream::check_upstream_http;

#[cfg(feature = "network")]
pub use upstream::UpstreamBackend;

// -----------------------------------------------------------------------------
// Imports
// -----------------------------------------------------------------------------

pub use crate::network::dns::reverse_dns_lookup;
pub use crate::network::dns::validate_dns_config;
pub use crate::network::ssl::check_ssl_url;
use crate::{ast::Config, Result};

// -----------------------------------------------------------------------------
// Unified result type
// -----------------------------------------------------------------------------

/// Unified result produced by high-level network checks.
///
/// All concrete network checks are normalized into this structure so
/// callers (CLI, API, CI) do not need to understand submodules.
#[derive(Debug, Clone)]
pub struct NetworkCheckResult {
    /// Category of check (dns, port, ssl, upstream)
    pub check_type: String,

    /// Target being checked (hostname, ip:port, path, etc.)
    pub target: String,

    /// Health status
    pub status: HealthStatus,

    /// Human-readable message
    pub message: String,

    /// Severity level
    pub severity: CheckSeverity,

    /// Optional extra details
    pub details: Option<String>,
}

// -----------------------------------------------------------------------------
// Top-level orchestration
// -----------------------------------------------------------------------------

/// Runs all enabled network checks against the configuration.
///
/// This is the **primary public entry point** for network validation.
/// It orchestrates DNS, port, SSL, and upstream checks based on the
/// provided options and returns normalized results.
///
/// # Arguments
///
/// * `config` - The parsed NGINX configuration to check
/// * `options` - Controls which checks to run and how to run them
///
/// # Returns
///
/// Returns a vector of `NetworkCheckResult` containing all check results.
/// The order of results is: ports, DNS, upstreams, SSL.
///
/// # Errors
///
/// Returns an error if:
/// - Server extraction from config fails
/// - A check encounters a fatal error
///
/// Note: Individual check failures are returned as results with error status,
/// not as `Err`.
///
/// # Examples
///
/// ```no_run
/// use nginx_discovery::{parse, network::{check_all, NetworkCheckOptions}};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = parse("server { listen 80; }")?;
///
///     // Check only ports and DNS
///     let options = NetworkCheckOptions {
///         check_ports: true,
///         check_dns: true,
///         check_ssl: false,
///         check_upstreams: false,
///         ..Default::default()
///     };
///
///     let results = check_all(&config, options).await?;
///     Ok(())
/// }
/// ```
pub async fn check_all(
    config: &Config,
    options: NetworkCheckOptions,
) -> Result<Vec<NetworkCheckResult>> {
    let mut results = Vec::new();

    if options.check_ports {
        results.extend(check_all_ports(config).await?);
    }

    if options.check_dns {
        results.extend(check_all_dns(config).await?);
    }

    // These are intentionally no-ops for now
    if options.check_upstreams {
        results.extend(check_all_upstreams(config).await?);
    }

    if options.check_ssl {
        results.extend(check_all_ssl(config).await?);
    }

    Ok(results)
}

// -----------------------------------------------------------------------------
// Upstream aggregation (stub – future-safe)
// -----------------------------------------------------------------------------

/// Checks all upstreams defined in the configuration.
///
/// # Implementation Note
///
/// Upstream extraction is not implemented yet. This function intentionally
/// returns an empty result set until `extract::upstreams` exists.
///
/// # Future Behavior
///
/// When implemented, this will:
/// - Extract all upstream blocks from the config
/// - Check connectivity to each backend
/// - Return health status for each upstream
#[allow(clippy::unused_async)]
async fn check_all_upstreams(_config: &Config) -> Result<Vec<NetworkCheckResult>> {
    // TODO:
    // Implement once upstream blocks are extracted from AST
    Ok(Vec::new())
}

// -----------------------------------------------------------------------------
// SSL aggregation (stub – future-safe)
// -----------------------------------------------------------------------------

/// Checks all SSL certificates referenced in the configuration.
///
/// # Implementation Note
///
/// Server SSL fields are not yet exposed in the AST. This is a placeholder
/// to preserve API stability.
///
/// # Future Behavior
///
/// When implemented, this will:
/// - Extract SSL certificate paths from server blocks
/// - Validate each certificate
/// - Check expiration dates
/// - Verify certificate chains
#[allow(clippy::unused_async)]
async fn check_all_ssl(_config: &Config) -> Result<Vec<NetworkCheckResult>> {
    // TODO:
    // Enable once Server exposes SSL certificate paths
    Ok(Vec::new())
}

// -----------------------------------------------------------------------------
// Port aggregation
// -----------------------------------------------------------------------------

/// Checks all listen directives for port availability.
///
/// Extracts all `listen` directives from server blocks and attempts
/// to connect to each port to verify it's accessible.
async fn check_all_ports(config: &Config) -> Result<Vec<NetworkCheckResult>> {
    #[cfg(not(feature = "network"))]
    {
        let _ = config;
        Ok(Vec::new())
    }

    #[cfg(feature = "network")]
    {
        use crate::extract::servers;

        let mut results = Vec::new();
        let servers = servers(config)?;

        for server in servers {
            for listen in &server.listen {
                match check_port(&listen.address, listen.port).await {
                    Ok(check) => results.push(NetworkCheckResult {
                        check_type: "port".to_string(),
                        target: format!("{}:{}", listen.address, listen.port),
                        status: check.status,
                        message: check.message,
                        severity: check.severity,
                        details: check.details,
                    }),
                    Err(e) => results.push(NetworkCheckResult {
                        check_type: "port".to_string(),
                        target: format!("{}:{}", listen.address, listen.port),
                        status: HealthStatus::Error,
                        message: format!("Port check failed: {e}"),
                        severity: CheckSeverity::Error,
                        details: None,
                    }),
                }
            }
        }

        Ok(results)
    }
}

// -----------------------------------------------------------------------------
// DNS aggregation
// -----------------------------------------------------------------------------

/// Checks DNS resolution for all server names.
///
/// Extracts all `server_name` directives and performs DNS resolution
/// for each hostname. Skips wildcards and special values like "_".
async fn check_all_dns(config: &Config) -> Result<Vec<NetworkCheckResult>> {
    #[cfg(not(feature = "network"))]
    {
        let _ = config;
        Ok(Vec::new())
    }

    #[cfg(feature = "network")]
    {
        use crate::extract::servers;

        let mut results = Vec::new();
        let servers = servers(config)?;

        for server in servers {
            for name in &server.server_names {
                // Skip wildcards and internal placeholders
                if name == "_" || name == "localhost" || name.contains('*') {
                    continue;
                }

                match resolve_hostname(name).await {
                    Ok(check) => results.push(NetworkCheckResult {
                        check_type: "dns".to_string(),
                        target: name.clone(),
                        status: check.status,
                        message: check.message,
                        severity: check.severity,
                        details: check.details,
                    }),
                    Err(e) => results.push(NetworkCheckResult {
                        check_type: "dns".to_string(),
                        target: name.clone(),
                        status: HealthStatus::Error,
                        message: format!("DNS resolution failed: {e}"),
                        severity: CheckSeverity::Warning,
                        details: None,
                    }),
                }
            }
        }

        Ok(results)
    }
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_check_options_default() {
        let options = NetworkCheckOptions::default();
        assert!(options.check_ports);
        assert!(options.check_dns);
        assert!(options.check_ssl);
        assert!(options.check_upstreams);
    }

    #[test]
    fn test_network_check_result_creation() {
        let result = NetworkCheckResult {
            check_type: "test".to_string(),
            target: "localhost:80".to_string(),
            status: HealthStatus::Healthy,
            message: "OK".to_string(),
            severity: CheckSeverity::Info,
            details: None,
        };

        assert_eq!(result.check_type, "test");
        assert_eq!(result.status, HealthStatus::Healthy);
    }
}
