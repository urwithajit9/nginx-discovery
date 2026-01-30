// src/network/dns.rs
//! DNS resolution and validation checking.
//!
//! This module provides asynchronous DNS resolution capabilities for checking
//! hostnames configured in NGINX. It supports parallel resolution and validation
//! with timeouts.
//!
//! # Examples
//!
//! ```no_run
//! use nginx_discovery::network::resolve_hostname;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let result = resolve_hostname("example.com").await?;
//!     println!("Resolved to: {:?}", result.addresses);
//!     Ok(())
//! }
//! ```

use super::types::{CheckSeverity, DnsCheckResult, DnsValidationResult, HealthStatus};
use crate::Result;
use std::time::{Duration, Instant};

/// Resolves a hostname to IP addresses with timeout.
///
/// Performs DNS resolution with a 5-second timeout. Returns detailed
/// information about the resolution including status, addresses found,
/// and resolution time.
///
/// # Arguments
///
/// * `hostname` - The hostname to resolve
///
/// # Returns
///
/// Returns a `DnsCheckResult` containing:
/// - Resolution status (Healthy, Unhealthy, Error)
/// - List of resolved IP addresses
/// - Resolution time
/// - Detailed error information if resolution failed
///
/// # Errors
///
/// Returns an error if:
/// - The `network` feature is not enabled
///
/// Note: DNS resolution failures are returned as `Ok(DnsCheckResult)` with
/// an error status, not as `Err`. Only feature-not-enabled returns `Err`.
///
/// # Examples
///
/// ```no_run
/// use nginx_discovery::network::resolve_hostname;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let result = resolve_hostname("google.com").await?;
///
///     if !result.addresses.is_empty() {
///         println!("Resolved to {} addresses", result.addresses.len());
///     }
///     Ok(())
/// }
/// ```
pub async fn resolve_hostname(hostname: &str) -> Result<DnsCheckResult> {
    #[cfg(feature = "network")]
    {
        use tokio::net::lookup_host;
        use tokio::time::timeout;

        let start = Instant::now();

        // Try to resolve with timeout
        let resolve_result =
            timeout(Duration::from_secs(5), lookup_host(format!("{hostname}:0"))).await;

        let resolution_time = start.elapsed();

        match resolve_result {
            Ok(Ok(addrs)) => {
                let addresses: Vec<String> = addrs.map(|addr| addr.ip().to_string()).collect();

                if addresses.is_empty() {
                    Ok(DnsCheckResult {
                        status: HealthStatus::Unhealthy,
                        message: format!("No addresses found for {hostname}"),
                        severity: CheckSeverity::Warning,
                        details: None,
                        hostname: hostname.to_string(),
                        addresses,
                        resolution_time: Some(resolution_time),
                    })
                } else {
                    Ok(DnsCheckResult {
                        status: HealthStatus::Healthy,
                        message: format!("Resolved {hostname} to {} address(es)", addresses.len()),
                        severity: CheckSeverity::Info,
                        details: Some(format!("Addresses: {}", addresses.join(", "))),
                        hostname: hostname.to_string(),
                        addresses,
                        resolution_time: Some(resolution_time),
                    })
                }
            }
            Ok(Err(e)) => Ok(DnsCheckResult {
                status: HealthStatus::Error,
                message: format!("Failed to resolve {hostname}"),
                severity: CheckSeverity::Error,
                details: Some(format!("Error: {e}")),
                hostname: hostname.to_string(),
                addresses: vec![],
                resolution_time: Some(resolution_time),
            }),
            Err(_) => Ok(DnsCheckResult {
                status: HealthStatus::Error,
                message: format!("Timeout resolving {hostname}"),
                severity: CheckSeverity::Warning,
                details: Some("Resolution timed out after 5 seconds".to_string()),
                hostname: hostname.to_string(),
                addresses: vec![],
                resolution_time: Some(resolution_time),
            }),
        }
    }

    #[cfg(not(feature = "network"))]
    {
        let _ = hostname;
        use crate::Error;
        Err(Error::FeatureNotEnabled("network".to_string()))
    }
}

/// Resolves multiple hostnames in parallel.
///
/// This function performs concurrent DNS resolution for multiple hostnames,
/// which is much faster than resolving them sequentially.
///
/// # Arguments
///
/// * `hostnames` - Vector of hostnames to resolve
///
/// # Returns
///
/// Returns a vector of `DnsCheckResult`, one for each hostname in the input.
/// Results are in the same order as the input hostnames.
///
/// # Errors
///
/// Returns an error if:
/// - The `network` feature is not enabled
///
/// Note: Individual DNS resolution failures are not returned as errors,
/// but as `DnsCheckResult` with error status.
///
/// # Examples
///
/// ```no_run
/// use nginx_discovery::network::resolve_hostnames;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let hostnames = vec![
///         "google.com".to_string(),
///         "github.com".to_string(),
///     ];
///
///     let results = resolve_hostnames(hostnames).await?;
///     for result in results {
///         println!("{}: {} addresses", result.hostname, result.addresses.len());
///     }
///     Ok(())
/// }
/// ```
pub async fn resolve_hostnames(hostnames: Vec<String>) -> Result<Vec<DnsCheckResult>> {
    #[cfg(feature = "network")]
    {
        use futures::future::join_all;

        // Clone hostname before borrowing to avoid lifetime issues
        let futures: Vec<_> = hostnames
            .iter()
            .map(|hostname| {
                let hostname_clone = hostname.clone();
                async move { resolve_hostname(&hostname_clone).await }
            })
            .collect();

        let results = join_all(futures).await;

        results.into_iter().collect()
    }

    #[cfg(not(feature = "network"))]
    {
        let _ = hostnames;
        use crate::Error;
        Err(Error::FeatureNotEnabled("network".to_string()))
    }
}

/// Performs reverse DNS lookup for an IP address.
///
/// # Arguments
///
/// * `_ip` - The IP address to look up
///
/// # Returns
///
/// Returns a vector of hostnames associated with the IP address.
///
/// # Errors
///
/// Returns an error if:
/// - The `network` feature is not enabled
///
/// # Implementation Note
///
/// This is currently a placeholder that returns an empty vector.
/// Full reverse DNS support will be added in a future version.
///
/// # Examples
///
/// ```no_run
/// use nginx_discovery::network::reverse_dns_lookup;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let hostnames = reverse_dns_lookup("8.8.8.8").await?;
/// # Ok(())
/// # }
/// ```
#[allow(clippy::unused_async)]
pub async fn reverse_dns_lookup(_ip: &str) -> Result<Vec<String>> {
    #[cfg(feature = "network")]
    {
        // TODO: Implement reverse DNS lookup
        // This will require additional DNS resolver configuration
        Ok(vec![])
    }

    #[cfg(not(feature = "network"))]
    {
        use crate::Error;
        Err(Error::FeatureNotEnabled("network".to_string()))
    }
}

/// Validates DNS configuration for a domain.
///
/// Performs basic DNS validation by checking if the domain resolves.
/// Future versions will include more comprehensive checks like NS records,
/// SOA records, and DNSSEC validation.
///
/// # Arguments
///
/// * `domain` - The domain name to validate
///
/// # Returns
///
/// Returns a `DnsValidationResult` containing:
/// - Whether the domain is valid (resolves successfully)
/// - NS records (future implementation)
/// - SOA record (future implementation)
///
/// # Errors
///
/// Returns an error if:
/// - The `network` feature is not enabled
/// - DNS resolution fails
///
/// # Examples
///
/// ```no_run
/// use nginx_discovery::network::validate_dns_config;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let result = validate_dns_config("example.com").await?;
///
///     if result.is_valid {
///         println!("Domain is valid");
///     }
///     Ok(())
/// }
/// ```
pub async fn validate_dns_config(domain: &str) -> Result<DnsValidationResult> {
    #[cfg(feature = "network")]
    {
        // Simplified version - just check if domain resolves
        let result = resolve_hostname(domain).await?;

        Ok(DnsValidationResult {
            domain: domain.to_string(),
            ns_records: None,
            soa_record: None,
            is_valid: !result.addresses.is_empty(),
        })
    }

    #[cfg(not(feature = "network"))]
    {
        let _ = domain;
        use crate::Error;
        Err(Error::FeatureNotEnabled("network".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(feature = "network")]
    async fn test_resolve_localhost() {
        let result = resolve_hostname("localhost").await;
        assert!(result.is_ok());

        let check = result.unwrap();
        assert_eq!(check.status, HealthStatus::Healthy);
        assert!(!check.addresses.is_empty());
    }

    #[tokio::test]
    #[cfg(feature = "network")]
    async fn test_resolve_invalid() {
        let result = resolve_hostname("this-domain-definitely-does-not-exist-12345.com").await;
        assert!(result.is_ok());

        let check = result.unwrap();
        assert_eq!(check.status, HealthStatus::Error);
    }

    #[tokio::test]
    #[cfg(feature = "network")]
    async fn test_resolve_multiple() {
        let hostnames = vec!["localhost".to_string()];
        let results = resolve_hostnames(hostnames).await;
        assert!(results.is_ok());

        let checks = results.unwrap();
        assert_eq!(checks.len(), 1);
    }

    #[tokio::test]
    #[cfg(feature = "network")]
    async fn test_validate_dns() {
        let result = validate_dns_config("localhost").await;
        assert!(result.is_ok());

        let validation = result.unwrap();
        assert!(validation.is_valid);
    }
}
