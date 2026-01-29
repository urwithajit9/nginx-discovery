// src/network/dns.rs
//! DNS resolution checking

use super::types::{DnsCheckResult, HealthStatus, CheckSeverity};
use crate::{Result, Error};
use std::time::{Duration, Instant};

/// Resolve hostname to IP addresses
///
/// # Examples
///
/// ```rust,no_run
/// use nginx_discovery::network::resolve_hostname;
///
/// #[tokio::main]
/// async fn main() {
///     let result = resolve_hostname("example.com").await;
///     match result {
///         Ok(check) => {
///             println!("Resolved to: {:?}", check.addresses);
///         }
///         Err(e) => eprintln!("Resolution failed: {}", e),
///     }
/// }
/// ```
pub async fn resolve_hostname(hostname: &str) -> Result<DnsCheckResult> {
    #[cfg(feature = "network")]
    {
        use tokio::net::lookup_host;
        use tokio::time::timeout;

        let start = Instant::now();

        // Try to resolve with timeout
        let resolve_result = timeout(
            Duration::from_secs(5),
            lookup_host(format!("{}:0", hostname))
        ).await;

        let resolution_time = start.elapsed();

        match resolve_result {
            Ok(Ok(addrs)) => {
                let addresses: Vec<String> = addrs
                    .map(|addr| addr.ip().to_string())
                    .collect();

                if addresses.is_empty() {
                    Ok(DnsCheckResult {
                        status: HealthStatus::Unhealthy,
                        message: format!("No addresses found for {}", hostname),
                        severity: CheckSeverity::Warning,
                        details: None,
                        hostname: hostname.to_string(),
                        addresses,
                        resolution_time: Some(resolution_time),
                    })
                } else {
                    Ok(DnsCheckResult {
                        status: HealthStatus::Healthy,
                        message: format!("Resolved {} to {} address(es)", hostname, addresses.len()),
                        severity: CheckSeverity::Info,
                        details: Some(format!("Addresses: {}", addresses.join(", "))),
                        hostname: hostname.to_string(),
                        addresses,
                        resolution_time: Some(resolution_time),
                    })
                }
            }
            Ok(Err(e)) => {
                Ok(DnsCheckResult {
                    status: HealthStatus::Error,
                    message: format!("Failed to resolve {}", hostname),
                    severity: CheckSeverity::Error,
                    details: Some(format!("Error: {}", e)),
                    hostname: hostname.to_string(),
                    addresses: vec![],
                    resolution_time: Some(resolution_time),
                })
            }
            Err(_) => {
                Ok(DnsCheckResult {
                    status: HealthStatus::Error,
                    message: format!("Timeout resolving {}", hostname),
                    severity: CheckSeverity::Warning,
                    details: Some("Resolution timed out after 5 seconds".to_string()),
                    hostname: hostname.to_string(),
                    addresses: vec![],
                    resolution_time: Some(resolution_time),
                })
            }
        }
    }

    #[cfg(not(feature = "network"))]
    {
        Err(Error::FeatureNotEnabled("network".to_string()))
    }
}

/// Resolve multiple hostnames in parallel
pub async fn resolve_hostnames(hostnames: Vec<String>) -> Result<Vec<DnsCheckResult>> {
    #[cfg(feature = "network")]
    {
        use futures::future::join_all;

        let futures: Vec<_> = hostnames
            .into_iter()
            .map(|hostname| resolve_hostname(&hostname))
            .collect();

        let results = join_all(futures).await;

        // Convert Vec<Result<DnsCheckResult>> to Result<Vec<DnsCheckResult>>
        results.into_iter().collect()
    }

    #[cfg(not(feature = "network"))]
    {
        Err(Error::FeatureNotEnabled("network".to_string()))
    }
}

/// Check reverse DNS (PTR record)
pub async fn reverse_dns_lookup(ip: &str) -> Result<Vec<String>> {
    #[cfg(feature = "network")]
    {
        use trust_dns_resolver::TokioAsyncResolver;
        use std::net::IpAddr;

        let ip_addr: IpAddr = ip.parse()
            .map_err(|e| Error::InvalidInput(format!("Invalid IP address: {}", e)))?;

        let resolver = TokioAsyncResolver::tokio_from_system_conf()
            .map_err(|e| Error::Network(format!("Failed to create resolver: {}", e)))?;

        let response = resolver.reverse_lookup(ip_addr)
            .await
            .map_err(|e| Error::Network(format!("Reverse lookup failed: {}", e)))?;

        let names: Vec<String> = response
            .iter()
            .map(|name| name.to_string())
            .collect();

        Ok(names)
    }

    #[cfg(not(feature = "network"))]
    {
        Err(Error::FeatureNotEnabled("network".to_string()))
    }
}

/// Validate DNS configuration (check NS records, SOA, etc.)
pub async fn validate_dns_config(domain: &str) -> Result<DnsValidationResult> {
    #[cfg(feature = "network")]
    {
        use trust_dns_resolver::TokioAsyncResolver;

        let resolver = TokioAsyncResolver::tokio_from_system_conf()
            .map_err(|e| Error::Network(format!("Failed to create resolver: {}", e)))?;

        // Check NS records
        let ns_records = resolver.ns_lookup(domain)
            .await
            .map(|response| {
                response.iter()
                    .map(|ns| ns.to_string())
                    .collect::<Vec<_>>()
            })
            .ok();

        // Check SOA record
        let soa_record = resolver.soa_lookup(domain)
            .await
            .ok()
            .and_then(|response| response.iter().next().map(|soa| soa.to_string()));

        Ok(DnsValidationResult {
            domain: domain.to_string(),
            ns_records,
            soa_record,
            is_valid: ns_records.is_some() || soa_record.is_some(),
        })
    }

    #[cfg(not(feature = "network"))]
    {
        Err(Error::FeatureNotEnabled("network".to_string()))
    }
}

/// DNS validation result
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DnsValidationResult {
    pub domain: String,
    pub ns_records: Option<Vec<String>>,
    pub soa_record: Option<String>,
    pub is_valid: bool,
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
}