// src/network/mod.rs
//! Network health checking and validation
//!
//! This module provides comprehensive network checking capabilities:
//! - Upstream backend health checks
//! - SSL/TLS certificate validation
//! - Port availability checks
//! - DNS resolution
//! - Network latency measurements

pub mod upstream;
pub mod ssl;
pub mod port;
pub mod dns;
pub mod types;

pub use types::{
    HealthStatus, HealthCheckResult, SslCheckResult, PortCheckResult, DnsCheckResult,
    CheckSeverity, NetworkCheckOptions,
};
pub use upstream::check_upstream;
pub use ssl::check_ssl_certificate;
pub use port::check_port;
pub use dns::resolve_hostname;

use crate::{Config, Result};

/// Run all network checks on a configuration
///
/// # Examples
///
/// ```rust,no_run
/// use nginx_discovery::network::check_all;
///
/// let config = nginx_discovery::parse(nginx_config)?;
/// let results = check_all(&config, Default::default())?;
///
/// for result in results {
///     println!("{}: {}", result.check_type, result.status);
/// }
/// ```
pub async fn check_all(config: &Config, options: NetworkCheckOptions) -> Result<Vec<NetworkCheckResult>> {
    let mut results = Vec::new();

    // Check upstreams
    if options.check_upstreams {
        let upstream_results = check_all_upstreams(config, &options).await?;
        results.extend(upstream_results);
    }

    // Check SSL certificates
    if options.check_ssl {
        let ssl_results = check_all_ssl(config, &options).await?;
        results.extend(ssl_results);
    }

    // Check ports
    if options.check_ports {
        let port_results = check_all_ports(config, &options).await?;
        results.extend(port_results);
    }

    // Check DNS
    if options.check_dns {
        let dns_results = check_all_dns(config, &options).await?;
        results.extend(dns_results);
    }

    Ok(results)
}

/// Unified network check result
#[derive(Debug, Clone)]
pub struct NetworkCheckResult {
    pub check_type: String,
    pub target: String,
    pub status: HealthStatus,
    pub message: String,
    pub severity: CheckSeverity,
    pub details: Option<String>,
}

/// Check all upstreams defined in config
async fn check_all_upstreams(config: &Config, options: &NetworkCheckOptions) -> Result<Vec<NetworkCheckResult>> {
    let mut results = Vec::new();

    // TODO: Extract upstreams from config
    // For now, return empty results

    Ok(results)
}

/// Check all SSL certificates
async fn check_all_ssl(config: &Config, options: &NetworkCheckOptions) -> Result<Vec<NetworkCheckResult>> {
    use crate::extract::servers;

    let mut results = Vec::new();
    let servers = servers(config)?;

    for server in servers {
        if let Some(ssl_config) = &server.ssl {
            if let Some(cert_path) = &ssl_config.certificate {
                match check_ssl_certificate(cert_path).await {
                    Ok(check_result) => {
                        results.push(NetworkCheckResult {
                            check_type: "ssl_certificate".to_string(),
                            target: cert_path.display().to_string(),
                            status: check_result.status,
                            message: check_result.message,
                            severity: check_result.severity,
                            details: check_result.details,
                        });
                    }
                    Err(e) => {
                        results.push(NetworkCheckResult {
                            check_type: "ssl_certificate".to_string(),
                            target: cert_path.display().to_string(),
                            status: HealthStatus::Error,
                            message: format!("Failed to check certificate: {}", e),
                            severity: CheckSeverity::Error,
                            details: None,
                        });
                    }
                }
            }
        }
    }

    Ok(results)
}

/// Check all ports
async fn check_all_ports(config: &Config, options: &NetworkCheckOptions) -> Result<Vec<NetworkCheckResult>> {
    use crate::extract::servers;

    let mut results = Vec::new();
    let servers = servers(config)?;

    for server in servers {
        for listen in &server.listen {
            match check_port(&listen.address, listen.port).await {
                Ok(check_result) => {
                    results.push(NetworkCheckResult {
                        check_type: "port_availability".to_string(),
                        target: format!("{}:{}", listen.address, listen.port),
                        status: check_result.status,
                        message: check_result.message,
                        severity: check_result.severity,
                        details: check_result.details,
                    });
                }
                Err(e) => {
                    results.push(NetworkCheckResult {
                        check_type: "port_availability".to_string(),
                        target: format!("{}:{}", listen.address, listen.port),
                        status: HealthStatus::Error,
                        message: format!("Failed to check port: {}", e),
                        severity: CheckSeverity::Error,
                        details: None,
                    });
                }
            }
        }
    }

    Ok(results)
}

/// Check all DNS entries
async fn check_all_dns(config: &Config, options: &NetworkCheckOptions) -> Result<Vec<NetworkCheckResult>> {
    use crate::extract::servers;

    let mut results = Vec::new();
    let servers = servers(config)?;

    for server in servers {
        for server_name in &server.server_names {
            // Skip wildcards and localhost
            if server_name.contains('*') || server_name == "localhost" || server_name == "_" {
                continue;
            }

            match resolve_hostname(server_name).await {
                Ok(check_result) => {
                    results.push(NetworkCheckResult {
                        check_type: "dns_resolution".to_string(),
                        target: server_name.clone(),
                        status: check_result.status,
                        message: check_result.message,
                        severity: check_result.severity,
                        details: check_result.details,
                    });
                }
                Err(e) => {
                    results.push(NetworkCheckResult {
                        check_type: "dns_resolution".to_string(),
                        target: server_name.clone(),
                        status: HealthStatus::Error,
                        message: format!("Failed to resolve: {}", e),
                        severity: CheckSeverity::Warning,
                        details: None,
                    });
                }
            }
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_check_structure() {
        let options = NetworkCheckOptions::default();
        assert!(options.check_upstreams);
        assert!(options.check_ssl);
    }
}