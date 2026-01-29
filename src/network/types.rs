// src/network/types.rs
//! Network check types and results

use std::time::Duration;

/// Health status of a check
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HealthStatus {
    /// Check passed successfully
    Healthy,

    /// Check passed with warnings
    Degraded,

    /// Check failed
    Unhealthy,

    /// Check could not be performed
    Error,

    /// Check not applicable
    NotApplicable,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => write!(f, "HEALTHY"),
            Self::Degraded => write!(f, "DEGRADED"),
            Self::Unhealthy => write!(f, "UNHEALTHY"),
            Self::Error => write!(f, "ERROR"),
            Self::NotApplicable => write!(f, "N/A"),
        }
    }
}

/// Severity of check result
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CheckSeverity {
    /// Informational only
    Info,

    /// Warning - should be addressed
    Warning,

    /// Error - needs immediate attention
    Error,

    /// Critical - system may be down
    Critical,
}

impl std::fmt::Display for CheckSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARNING"),
            Self::Error => write!(f, "ERROR"),
            Self::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Generic health check result
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub message: String,
    pub severity: CheckSeverity,
    pub details: Option<String>,
    pub latency: Option<Duration>,
}

impl HealthCheckResult {
    pub fn healthy(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Healthy,
            message: message.into(),
            severity: CheckSeverity::Info,
            details: None,
            latency: None,
        }
    }

    pub fn degraded(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Degraded,
            message: message.into(),
            severity: CheckSeverity::Warning,
            details: None,
            latency: None,
        }
    }

    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            message: message.into(),
            severity: CheckSeverity::Error,
            details: None,
            latency: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Error,
            message: message.into(),
            severity: CheckSeverity::Critical,
            details: None,
            latency: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    pub fn with_latency(mut self, latency: Duration) -> Self {
        self.latency = Some(latency);
        self
    }
}

/// SSL certificate check result
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SslCheckResult {
    pub status: HealthStatus,
    pub message: String,
    pub severity: CheckSeverity,
    pub details: Option<String>,

    /// Certificate expiry date
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Days until expiration
    pub days_until_expiry: Option<i64>,

    /// Certificate issuer
    pub issuer: Option<String>,

    /// Certificate subject
    pub subject: Option<String>,
}

/// Port availability check result
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PortCheckResult {
    pub status: HealthStatus,
    pub message: String,
    pub severity: CheckSeverity,
    pub details: Option<String>,

    /// Port number
    pub port: u16,

    /// Address
    pub address: String,

    /// Whether port is listening
    pub is_listening: bool,

    /// Connection latency
    pub latency: Option<Duration>,
}

/// DNS resolution check result
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DnsCheckResult {
    pub status: HealthStatus,
    pub message: String,
    pub severity: CheckSeverity,
    pub details: Option<String>,

    /// Hostname being resolved
    pub hostname: String,

    /// Resolved IP addresses
    pub addresses: Vec<String>,

    /// Resolution time
    pub resolution_time: Option<Duration>,
}

/// Options for network checks
#[derive(Debug, Clone)]
pub struct NetworkCheckOptions {
    /// Check upstream backends
    pub check_upstreams: bool,

    /// Check SSL certificates
    pub check_ssl: bool,

    /// Check port availability
    pub check_ports: bool,

    /// Check DNS resolution
    pub check_dns: bool,

    /// Timeout for each check
    pub timeout: Duration,

    /// Number of retries
    pub retries: usize,

    /// Parallel execution
    pub parallel: bool,

    /// Continue on error
    pub continue_on_error: bool,
}

impl Default for NetworkCheckOptions {
    fn default() -> Self {
        Self {
            check_upstreams: true,
            check_ssl: true,
            check_ports: true,
            check_dns: true,
            timeout: Duration::from_secs(5),
            retries: 3,
            parallel: true,
            continue_on_error: true,
        }
    }
}

impl NetworkCheckOptions {
    /// Create options that only check upstreams
    pub fn upstreams_only() -> Self {
        Self {
            check_upstreams: true,
            check_ssl: false,
            check_ports: false,
            check_dns: false,
            ..Default::default()
        }
    }

    /// Create options that only check SSL
    pub fn ssl_only() -> Self {
        Self {
            check_upstreams: false,
            check_ssl: true,
            check_ports: false,
            check_dns: false,
            ..Default::default()
        }
    }

    /// Create options that only check ports
    pub fn ports_only() -> Self {
        Self {
            check_upstreams: false,
            check_ssl: false,
            check_ports: true,
            check_dns: false,
            ..Default::default()
        }
    }

    /// Create options that only check DNS
    pub fn dns_only() -> Self {
        Self {
            check_upstreams: false,
            check_ssl: false,
            check_ports: false,
            check_dns: true,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_display() {
        assert_eq!(HealthStatus::Healthy.to_string(), "HEALTHY");
        assert_eq!(HealthStatus::Unhealthy.to_string(), "UNHEALTHY");
    }

    #[test]
    fn test_severity_ordering() {
        assert!(CheckSeverity::Info < CheckSeverity::Warning);
        assert!(CheckSeverity::Warning < CheckSeverity::Error);
        assert!(CheckSeverity::Error < CheckSeverity::Critical);
    }

    #[test]
    fn test_health_check_builders() {
        let result = HealthCheckResult::healthy("All good")
            .with_details("Everything is working");

        assert_eq!(result.status, HealthStatus::Healthy);
        assert_eq!(result.message, "All good");
        assert!(result.details.is_some());
    }

    #[test]
    fn test_network_options_presets() {
        let ssl_only = NetworkCheckOptions::ssl_only();
        assert!(!ssl_only.check_upstreams);
        assert!(ssl_only.check_ssl);
        assert!(!ssl_only.check_ports);
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