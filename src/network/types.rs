//! Network check types and results
//!
//! This module defines the **core data structures** used by the network
//! health-checking subsystem.
//!
//! ## Scope and responsibilities
//!
//! - Types in this module are **pure data containers**
//! - No I/O, networking, async, or side effects are performed here
//! - All execution logic lives in sibling modules (e.g. `ssl`, `dns`, `upstream`)
//!
//! ## Design principles
//!
//! - All check results follow a **normalized shape**
//! - Severity and health status are explicitly modeled
//! - Structures are optimized for:
//!   - CLI output
//!   - JSON serialization
//!   - Aggregation and reporting
//!
//! This mirrors how large frameworks (e.g. Kubernetes, Django system checks)
//! separate **evaluation** from **representation**.

use std::time::Duration;

/* ============================================================
 * Health status & severity
 * ============================================================
 */

/// Overall health status of a network check.
///
/// This represents **what happened**, independent of urgency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HealthStatus {
    /// Check succeeded with no issues.
    Healthy,

    /// Check succeeded but non-fatal issues were detected.
    Degraded,

    /// Check failed and the system is unhealthy.
    Unhealthy,

    /// Check could not be executed due to an internal error.
    Error,

    /// Check was intentionally skipped or not applicable.
    NotApplicable,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Healthy => "HEALTHY",
            Self::Degraded => "DEGRADED",
            Self::Unhealthy => "UNHEALTHY",
            Self::Error => "ERROR",
            Self::NotApplicable => "N/A",
        };
        write!(f, "{s}")
    }
}

/// Severity level associated with a health-check result.
///
/// This represents **how urgent** the outcome is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CheckSeverity {
    /// Informational only; no action required.
    Info,

    /// Warning; action recommended.
    Warning,

    /// Error; action required.
    Error,

    /// Critical; immediate action required.
    Critical,
}

impl std::fmt::Display for CheckSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Info => "INFO",
            Self::Warning => "WARNING",
            Self::Error => "ERROR",
            Self::Critical => "CRITICAL",
        };
        write!(f, "{s}")
    }
}

/* ============================================================
 * Generic health check
 * ============================================================
 */

/// Normalized result type used by **all** network checks.
///
/// This type allows heterogeneous checks (SSL, DNS, ports, upstreams)
/// to be aggregated and displayed uniformly.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HealthCheckResult {
    /// Final health status.
    pub status: HealthStatus,

    /// Human-readable summary message.
    pub message: String,

    /// Severity level.
    pub severity: CheckSeverity,

    /// Optional diagnostic details.
    pub details: Option<String>,

    /// Optional latency measurement.
    pub latency: Option<Duration>,
}

impl HealthCheckResult {
    /// Create a healthy result.
    #[must_use]
    pub fn healthy(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Healthy,
            message: message.into(),
            severity: CheckSeverity::Info,
            details: None,
            latency: None,
        }
    }

    /// Create a degraded result.
    #[must_use]
    pub fn degraded(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Degraded,
            message: message.into(),
            severity: CheckSeverity::Warning,
            details: None,
            latency: None,
        }
    }

    /// Create an unhealthy result.
    #[must_use]
    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            message: message.into(),
            severity: CheckSeverity::Error,
            details: None,
            latency: None,
        }
    }

    /// Create an error result.
    #[must_use]
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Error,
            message: message.into(),
            severity: CheckSeverity::Critical,
            details: None,
            latency: None,
        }
    }

    /// Attach diagnostic details.
    #[must_use]
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Attach latency information.
    #[must_use]
    pub fn with_latency(mut self, latency: Duration) -> Self {
        self.latency = Some(latency);
        self
    }
}

/* ============================================================
 * SSL check
 * ============================================================
 */

/// Result of an SSL/TLS certificate validation.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SslCheckResult {
    /// Health status of the SSL check.
    pub status: HealthStatus,

    /// Summary message.
    pub message: String,

    /// Severity level.
    pub severity: CheckSeverity,

    /// Optional diagnostic details.
    pub details: Option<String>,

    /// Certificate expiration timestamp (UTC).
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Days remaining until certificate expiration.
    pub days_until_expiry: Option<i64>,

    /// Certificate issuer (distinguished name).
    pub issuer: Option<String>,

    /// Certificate subject (distinguished name).
    pub subject: Option<String>,
}

/* ============================================================
 * Port check
 * ============================================================
 */

/// Result of a TCP port availability check.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PortCheckResult {
    /// Health status of the port check.
    pub status: HealthStatus,

    /// Summary message.
    pub message: String,

    /// Severity level.
    pub severity: CheckSeverity,

    /// Optional diagnostic details.
    pub details: Option<String>,

    /// Port number that was checked.
    pub port: u16,

    /// Hostname or IP address.
    pub address: String,

    /// Whether the port is accepting connections.
    pub is_listening: bool,

    /// Optional connection latency.
    pub latency: Option<Duration>,
}

/* ============================================================
 * DNS resolution
 * ============================================================
 */

/// Result of a DNS resolution check.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DnsCheckResult {
    /// Health status of the DNS check.
    pub status: HealthStatus,

    /// Summary message.
    pub message: String,

    /// Severity level.
    pub severity: CheckSeverity,

    /// Optional diagnostic details.
    pub details: Option<String>,

    /// Hostname that was resolved.
    pub hostname: String,

    /// Resolved IP addresses.
    pub addresses: Vec<String>,

    /// Time taken to resolve DNS.
    pub resolution_time: Option<Duration>,
}

/* ============================================================
 * Network execution options
 * ============================================================
 */

/// Configuration controlling which network checks are executed.
///
/// This struct is intentionally explicit rather than compact, as
/// configuration clarity is preferred over minimalism.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct NetworkCheckOptions {
    /// Whether to check upstream backends.
    pub check_upstreams: bool,

    /// Whether to validate SSL certificates.
    pub check_ssl: bool,

    /// Whether to check TCP ports.
    pub check_ports: bool,

    /// Whether to perform DNS resolution.
    pub check_dns: bool,

    /// Per-check timeout.
    pub timeout: Duration,

    /// Number of retry attempts.
    pub retries: usize,

    /// Execute checks concurrently.
    pub parallel: bool,

    /// Continue executing checks after failures.
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
    /// Enable only upstream checks.
    #[must_use]
    pub fn upstreams_only() -> Self {
        Self {
            check_upstreams: true,
            check_ssl: false,
            check_ports: false,
            check_dns: false,
            ..Default::default()
        }
    }

    /// Enable only SSL checks.
    #[must_use]
    pub fn ssl_only() -> Self {
        Self {
            check_upstreams: false,
            check_ssl: true,
            check_ports: false,
            check_dns: false,
            ..Default::default()
        }
    }

    /// Enable only port checks.
    #[must_use]
    pub fn ports_only() -> Self {
        Self {
            check_upstreams: false,
            check_ssl: false,
            check_ports: true,
            check_dns: false,
            ..Default::default()
        }
    }

    /// Enable only DNS checks.
    #[must_use]
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

/* ============================================================
 * DNS validation
 * ============================================================
 */

/// Result of DNS configuration validation.
///
/// This is distinct from DNS *resolution* and focuses on correctness
/// of authoritative records.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DnsValidationResult {
    /// Domain name that was validated.
    pub domain: String,

    /// NS records discovered for the domain.
    pub ns_records: Option<Vec<String>>,

    /// SOA record, if available.
    pub soa_record: Option<String>,

    /// Whether the DNS configuration is valid.
    pub is_valid: bool,
}
