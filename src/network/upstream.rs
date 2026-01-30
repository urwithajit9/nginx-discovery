//! Upstream backend health checking
//!
//! This module provides primitives for checking the health of upstream backends
//! (TCP and HTTP), aggregating results across backend groups, and calculating
//! overall upstream health percentages.

use super::types::{CheckSeverity, HealthCheckResult, HealthStatus};
use crate::{Error, Result};
use std::time::{Duration, Instant};

/// Upstream backend definition
///
/// Represents a single backend server in an upstream pool.
#[derive(Debug, Clone)]
pub struct UpstreamBackend {
    /// The hostname or IP address of the backend server
    pub host: String,

    /// The port number the backend server is listening on
    pub port: u16,

    /// Optional weight for load balancing
    pub weight: Option<u32>,

    /// Optional maximum number of failed attempts
    pub max_fails: Option<u32>,

    /// Optional duration to wait before retrying a failed backend
    pub fail_timeout: Option<Duration>,
}

/// Perform a TCP health check against a single upstream backend.
///
/// # Errors
///
/// Returns an error if the `network` feature is disabled.
pub async fn check_upstream_backend(backend: &UpstreamBackend) -> Result<HealthCheckResult> {
    #[cfg(feature = "network")]
    {
        use tokio::net::TcpStream;
        use tokio::time::timeout;

        let target = format!("{host}:{port}", host = backend.host, port = backend.port);
        let start = Instant::now();

        let connect_result = timeout(Duration::from_secs(5), TcpStream::connect(&target)).await;

        let latency = start.elapsed();

        match connect_result {
            Ok(Ok(_)) => {
                let (status, severity) = if latency > Duration::from_secs(2) {
                    (HealthStatus::Degraded, CheckSeverity::Warning)
                } else {
                    (HealthStatus::Healthy, CheckSeverity::Info)
                };

                Ok(HealthCheckResult {
                    status,
                    message: format!("Backend {target} is reachable"),
                    severity,
                    details: Some(format!("Response time: {latency:?}")),
                    latency: Some(latency),
                })
            }
            Ok(Err(e)) => Ok(HealthCheckResult {
                status: HealthStatus::Unhealthy,
                message: format!("Backend {target} is unreachable"),
                severity: CheckSeverity::Error,
                details: Some(format!("Connection failed: {e}")),
                latency: Some(latency),
            }),
            Err(_) => Ok(HealthCheckResult {
                status: HealthStatus::Error,
                message: format!("Backend {target} timed out"),
                severity: CheckSeverity::Critical,
                details: Some("Connection timed out after 5 seconds".to_string()),
                latency: Some(latency),
            }),
        }
    }

    #[cfg(not(feature = "network"))]
    {
        Err(Error::FeatureNotEnabled("network".to_string()))
    }
}

/// Perform an HTTP health check against an upstream backend.
///
/// # Errors
///
/// Returns an error if:
/// - The `network` feature is disabled
/// - The HTTP client cannot be constructed
pub async fn check_upstream_http(
    backend: &UpstreamBackend,
    health_check_path: &str,
) -> Result<HealthCheckResult> {
    #[cfg(feature = "network")]
    {
        let url = format!(
            "http://{host}:{port}{path}",
            host = backend.host,
            port = backend.port,
            path = health_check_path
        );

        let start = Instant::now();

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| Error::Network(format!("Failed to create client: {e}")))?;

        match client.get(&url).send().await {
            Ok(response) => {
                let latency = start.elapsed();
                let status_code = response.status();

                let (status, severity) = if status_code.is_success() {
                    (HealthStatus::Healthy, CheckSeverity::Info)
                } else if status_code.is_server_error() {
                    (HealthStatus::Unhealthy, CheckSeverity::Error)
                } else {
                    (HealthStatus::Degraded, CheckSeverity::Warning)
                };

                Ok(HealthCheckResult {
                    status,
                    message: format!("Backend returned status {status_code}"),
                    severity,
                    details: Some(format!("Response time: {latency:?}, Status: {status_code}")),
                    latency: Some(latency),
                })
            }
            Err(e) => {
                let latency = start.elapsed();
                Ok(HealthCheckResult {
                    status: HealthStatus::Unhealthy,
                    message: format!("HTTP check failed for {url}"),
                    severity: CheckSeverity::Error,
                    details: Some(format!("Error: {e}")),
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

/// Perform health checks for all backends in an upstream group.
///
/// # Errors
///
/// Returns an error if the `network` feature is disabled.
pub async fn check_upstream_group(
    backends: Vec<UpstreamBackend>,
) -> Result<Vec<HealthCheckResult>> {
    #[cfg(feature = "network")]
    {
        use futures::future::join_all;

        let futures = backends.iter().map(check_upstream_backend);
        let results = join_all(futures).await;

        results.into_iter().collect()
    }

    #[cfg(not(feature = "network"))]
    {
        Err(Error::FeatureNotEnabled("network".to_string()))
    }
}

/// Calculate upstream group health percentage.
///
/// Returns a value between `0.0` and `100.0`.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn calculate_group_health(results: &[HealthCheckResult]) -> f64 {
    if results.is_empty() {
        return 0.0;
    }

    let healthy = results
        .iter()
        .filter(|r| r.status == HealthStatus::Healthy)
        .count();

    (healthy as f64 / results.len() as f64) * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_group_health() {
        let results = vec![
            HealthCheckResult::healthy("OK"),
            HealthCheckResult::healthy("OK"),
            HealthCheckResult::unhealthy("Down"),
        ];

        let health = calculate_group_health(&results);
        assert!((health - 66.67).abs() < 0.1);
    }

    #[test]
    fn test_empty_group_health() {
        let results = vec![];
        assert_eq!(calculate_group_health(&results), 0.0);
    }
}
