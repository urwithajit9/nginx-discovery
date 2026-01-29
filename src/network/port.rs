// src/network/port.rs
//! Port availability checking

use super::types::{PortCheckResult, HealthStatus, CheckSeverity};
use crate::{Result, Error};
use std::time::{Duration, Instant};

/// Check if a port is available (listening)
///
/// # Examples
///
/// ```rust,no_run
/// use nginx_discovery::network::check_port;
///
/// #[tokio::main]
/// async fn main() {
///     let result = check_port("127.0.0.1", 80).await;
///     match result {
///         Ok(check) => {
///             if check.is_listening {
///                 println!("Port 80 is listening");
///             }
///         }
///         Err(e) => eprintln!("Check failed: {}", e),
///     }
/// }
/// ```
pub async fn check_port(address: &str, port: u16) -> Result<PortCheckResult> {
    #[cfg(feature = "network")]
    {
        use tokio::net::TcpStream;
        use tokio::time::timeout;

        let target = format!("{}:{}", address, port);
        let start = Instant::now();

        // Try to connect with timeout
        let connect_result = timeout(
            Duration::from_secs(5),
            TcpStream::connect(&target)
        ).await;

        let latency = start.elapsed();

        match connect_result {
            Ok(Ok(_stream)) => {
                Ok(PortCheckResult {
                    status: HealthStatus::Healthy,
                    message: format!("Port {} is listening on {}", port, address),
                    severity: CheckSeverity::Info,
                    details: Some(format!("Connection established in {:?}", latency)),
                    port,
                    address: address.to_string(),
                    is_listening: true,
                    latency: Some(latency),
                })
            }
            Ok(Err(e)) => {
                Ok(PortCheckResult {
                    status: HealthStatus::Unhealthy,
                    message: format!("Port {} is not listening on {}", port, address),
                    severity: CheckSeverity::Error,
                    details: Some(format!("Connection failed: {}", e)),
                    port,
                    address: address.to_string(),
                    is_listening: false,
                    latency: Some(latency),
                })
            }
            Err(_) => {
                Ok(PortCheckResult {
                    status: HealthStatus::Error,
                    message: format!("Timeout checking port {} on {}", port, address),
                    severity: CheckSeverity::Warning,
                    details: Some("Connection timed out after 5 seconds".to_string()),
                    port,
                    address: address.to_string(),
                    is_listening: false,
                    latency: Some(latency),
                })
            }
        }
    }

    #[cfg(not(feature = "network"))]
    {
        Err(Error::FeatureNotEnabled("network".to_string()))
    }
}

/// Check multiple ports in parallel
pub async fn check_ports(addresses: Vec<(String, u16)>) -> Result<Vec<PortCheckResult>> {
    #[cfg(feature = "network")]
    {
        use futures::future::join_all;

        let futures: Vec<_> = addresses
            .into_iter()
            .map(|(addr, port)| check_port(&addr, port))
            .collect();

        let results = join_all(futures).await;

        // Convert Vec<Result<PortCheckResult>> to Result<Vec<PortCheckResult>>
        results.into_iter().collect()
    }

    #[cfg(not(feature = "network"))]
    {
        Err(Error::FeatureNotEnabled("network".to_string()))
    }
}

/// Check if port is in use (for server startup validation)
pub async fn is_port_in_use(port: u16) -> Result<bool> {
    match check_port("127.0.0.1", port).await {
        Ok(result) => Ok(result.is_listening),
        Err(_) => Ok(false),
    }
}

/// Find available port in range
pub async fn find_available_port(start: u16, end: u16) -> Result<Option<u16>> {
    #[cfg(feature = "network")]
    {
        for port in start..=end {
            if !is_port_in_use(port).await? {
                return Ok(Some(port));
            }
        }
        Ok(None)
    }

    #[cfg(not(feature = "network"))]
    {
        Err(Error::FeatureNotEnabled("network".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(feature = "network")]
    async fn test_check_localhost_port() {
        // Check a port that's likely not in use
        let result = check_port("127.0.0.1", 59999).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[cfg(feature = "network")]
    async fn test_is_port_in_use() {
        // Should not be in use
        let result = is_port_in_use(59999).await;
        assert!(result.is_ok());
    }
}