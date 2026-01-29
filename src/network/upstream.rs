// src/network/upstream.rs
//! Upstream backend health checking

use super::types::{HealthCheckResult, HealthStatus, CheckSeverity};
use crate::{Result, Error};
use std::time::{Duration, Instant};

/// Upstream backend definition
#[derive(Debug, Clone)]
pub struct UpstreamBackend {
    pub host: String,
    pub port: u16,
    pub weight: Option<u32>,
    pub max_fails: Option<u32>,
    pub fail_timeout: Option<Duration>,
}

/// Check upstream backend health
pub async fn check_upstream_backend(backend: &UpstreamBackend) -> Result<HealthCheckResult> {
    #[cfg(feature = "network")]
    {
        use tokio::net::TcpStream;
        use tokio::time::timeout;

        let target = format!("{}:{}", backend.host, backend.port);
        let start = Instant::now();

        // Try to connect
        let connect_result = timeout(
            Duration::from_secs(5),
            TcpStream::connect(&target)
        ).await;

        let latency = start.elapsed();

        match connect_result {
            Ok(Ok(_stream)) => {
                let status = if latency > Duration::from_secs(2) {
                    HealthStatus::Degraded
                } else {
                    HealthStatus::Healthy
                };

                let severity = if latency > Duration::from_secs(2) {
                    CheckSeverity::Warning
                } else {
                    CheckSeverity::Info
                };

                Ok(HealthCheckResult {
                    status,
                    message: format!("Backend {} is reachable", target),
                    severity,
                    details: Some(format!("Response time: {:?}", latency)),
                    latency: Some(latency),
                })
            }
            Ok(Err(e)) => {
                Ok(HealthCheckResult {
                    status: HealthStatus::Unhealthy,
                    message: format!("Backend {} is unreachable", target),
                    severity: CheckSeverity::Error,
                    details: Some(format!("Connection failed: {}", e)),
                    latency: Some(latency),
                })
            }
            Err(_) => {
                Ok(HealthCheckResult {
                    status: HealthStatus::Error,
                    message: format!("Backend {} timed out", target),
                    severity: CheckSeverity::Critical,
                    details: Some("Connection timed out after 5 seconds".to_string()),
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

/// Check upstream with HTTP health check
pub async fn check_upstream_http(
    backend: &UpstreamBackend,
    health_check_path: &str,
) -> Result<HealthCheckResult> {
    #[cfg(feature = "network")]
    {
        use reqwest;

        let url = format!("http://{}:{}{}", backend.host, backend.port, health_check_path);
        let start = Instant::now();

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| Error::Network(format!("Failed to create client: {}", e)))?;

        match client.get(&url).send().await {
            Ok(response) => {
                let latency = start.elapsed();
                let status_code = response.status();

                let (health_status, severity, message) = if status_code.is_success() {
                    (
                        HealthStatus::Healthy,
                        CheckSeverity::Info,
                        format!("Backend returned status {}", status_code),
                    )
                } else if status_code.is_server_error() {
                    (
                        HealthStatus::Unhealthy,
                        CheckSeverity::Error,
                        format!("Backend returned error status {}", status_code),
                    )
                } else {
                    (
                        HealthStatus::Degraded,
                        CheckSeverity::Warning,
                        format!("Backend returned status {}", status_code),
                    )
                };

                Ok(HealthCheckResult {
                    status: health_status,
                    message,
                    severity,
                    details: Some(format!("Response time: {:?}, Status: {}", latency, status_code)),
                    latency: Some(latency),
                })
            }
            Err(e) => {
                let latency = start.elapsed();
                Ok(HealthCheckResult {
                    status: HealthStatus::Unhealthy,
                    message: format!("HTTP check failed for {}", url),
                    severity: CheckSeverity::Error,
                    details: Some(format!("Error: {}", e)),
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

/// Check all backends in an upstream group
pub async fn check_upstream_group(backends: Vec<UpstreamBackend>) -> Result<Vec<HealthCheckResult>> {
    #[cfg(feature = "network")]
    {
        use futures::future::join_all;

        let futures: Vec<_> = backends
            .iter()
            .map(|backend| check_upstream_backend(backend))
            .collect();

        let results = join_all(futures).await;

        // Convert Vec<Result<HealthCheckResult>> to Result<Vec<HealthCheckResult>>
        results.into_iter().collect()
    }

    #[cfg(not(feature = "network"))]
    {
        Err(Error::FeatureNotEnabled("network".to_string()))
    }
}

/// Calculate upstream group health percentage
pub fn calculate_group_health(results: &[HealthCheckResult]) -> f64 {
    if results.is_empty() {
        return 0.0;
    }

    let healthy_count = results
        .iter()
        .filter(|r| r.status == HealthStatus::Healthy)
        .count();

    (healthy_count as f64 / results.len() as f64) * 100.0
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
        let health = calculate_group_health(&results);
        assert_eq!(health, 0.0);
    }
}