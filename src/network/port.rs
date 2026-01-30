//! Port availability checking
//!
//! Provides utilities for checking whether TCP ports are listening,
//! scanning multiple ports concurrently, and finding available ports.

use super::types::{CheckSeverity, HealthStatus, PortCheckResult};
use crate::Result;
use std::time::{Duration, Instant};

/// Check if a port is available (listening).
///
/// Performs a TCP connection attempt to the given address and port.
///
/// # Errors
///
/// Returns an error if the `network` feature is disabled.
pub async fn check_port(address: &str, port: u16) -> Result<PortCheckResult> {
    #[cfg(feature = "network")]
    {
        use tokio::net::TcpStream;
        use tokio::time::timeout;

        let target = format!("{address}:{port}");
        let start = Instant::now();

        let connect_result = timeout(Duration::from_secs(5), TcpStream::connect(&target)).await;

        let latency = start.elapsed();

        match connect_result {
            Ok(Ok(_)) => Ok(PortCheckResult {
                status: HealthStatus::Healthy,
                message: format!("Port {port} is listening on {address}"),
                severity: CheckSeverity::Info,
                details: Some(format!("Connection established in {latency:?}")),
                port,
                address: address.to_string(),
                is_listening: true,
                latency: Some(latency),
            }),
            Ok(Err(e)) => Ok(PortCheckResult {
                status: HealthStatus::Unhealthy,
                message: format!("Port {port} is not listening on {address}"),
                severity: CheckSeverity::Error,
                details: Some(format!("Connection failed: {e}")),
                port,
                address: address.to_string(),
                is_listening: false,
                latency: Some(latency),
            }),
            Err(_) => Ok(PortCheckResult {
                status: HealthStatus::Error,
                message: format!("Timeout checking port {port} on {address}"),
                severity: CheckSeverity::Warning,
                details: Some("Connection timed out after 5 seconds".to_string()),
                port,
                address: address.to_string(),
                is_listening: false,
                latency: Some(latency),
            }),
        }
    }

    #[cfg(not(feature = "network"))]
    {
        use crate::Error;
        Err(Error::FeatureNotEnabled("network".to_string()))
    }
}

/// Check multiple ports concurrently.
///
/// # Errors
///
/// Returns an error if the `network` feature is disabled.
pub async fn check_ports(addresses: Vec<(String, u16)>) -> Result<Vec<PortCheckResult>> {
    #[cfg(feature = "network")]
    {
        use futures::future::join_all;

        let futures: Vec<_> = addresses
            .iter()
            .map(|(addr, port)| {
                let addr = addr.clone();
                let port = *port;
                async move { check_port(&addr, port).await }
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

/// Check if a local port is currently in use.
///
/// # Errors
///
/// Returns an error if the underlying port check fails.
pub async fn is_port_in_use(port: u16) -> Result<bool> {
    match check_port("127.0.0.1", port).await {
        Ok(result) => Ok(result.is_listening),
        Err(_) => Ok(false),
    }
}

/// Find the first available port within a range.
///
/// # Errors
///
/// Returns an error if the `network` feature is disabled
/// or if a port check fails.
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
        use crate::Error;
        Err(Error::FeatureNotEnabled("network".to_string()))
    }
}
