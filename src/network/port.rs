// src/network/dns.rs
//! DNS resolution checking

use super::types::{DnsCheckResult, HealthStatus, CheckSeverity, DnsValidationResult};
use crate::Result;
use std::time::{Duration, Instant};

/// Resolve hostname to IP addresses
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
        use crate::Error;
        Err(Error::FeatureNotEnabled("network".to_string()))
    }
}

/// Resolve multiple hostnames in parallel - FIXED BORROWING
pub async fn resolve_hostnames(hostnames: Vec<String>) -> Result<Vec<DnsCheckResult>> {
    #[cfg(feature = "network")]
    {
        use futures::future::join_all;

        // Fix: Clone hostname before borrowing
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
        use crate::Error;
        Err(Error::FeatureNotEnabled("network".to_string()))
    }
}

/// Check reverse DNS (PTR record) - Simplified version
pub async fn reverse_dns_lookup(_ip: &str) -> Result<Vec<String>> {
    #[cfg(feature = "network")]
    {
        // Simplified - not fully implemented yet
        Ok(vec![])
    }

    #[cfg(not(feature = "network"))]
    {
        use crate::Error;
        Err(Error::FeatureNotEnabled("network".to_string()))
    }
}

/// Validate DNS configuration - Simplified version
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
}