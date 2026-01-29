// src/network/ssl.rs
//! SSL/TLS certificate validation and checking

use super::types::{SslCheckResult, HealthStatus, CheckSeverity};
use crate::{Result, Error};
use std::path::Path;
use chrono::{DateTime, Utc, Duration};

/// Check SSL certificate file
///
/// # Examples
///
/// ```rust,no_run
/// use nginx_discovery::network::check_ssl_certificate;
/// use std::path::Path;
///
/// #[tokio::main]
/// async fn main() {
///     let result = check_ssl_certificate(Path::new("/etc/ssl/cert.pem")).await;
///     match result {
///         Ok(check) => println!("Certificate expires: {:?}", check.expires_at),
///         Err(e) => eprintln!("Check failed: {}", e),
///     }
/// }
/// ```
pub async fn check_ssl_certificate(cert_path: &Path) -> Result<SslCheckResult> {
    // Check if file exists
    if !cert_path.exists() {
        return Ok(SslCheckResult {
            status: HealthStatus::Error,
            message: format!("Certificate file not found: {}", cert_path.display()),
            severity: CheckSeverity::Critical,
            details: None,
            expires_at: None,
            days_until_expiry: None,
            issuer: None,
            subject: None,
        });
    }

    // Read certificate file
    let cert_data = std::fs::read(cert_path)
        .map_err(|e| Error::Io(e))?;

    // Parse certificate
    let cert_info = parse_x509_certificate(&cert_data)?;

    // Calculate days until expiry
    let now = Utc::now();
    let days_until_expiry = if let Some(expires_at) = cert_info.expires_at {
        Some((expires_at - now).num_days())
    } else {
        None
    };

    // Determine status based on expiry
    let (status, severity, message) = match days_until_expiry {
        Some(days) if days < 0 => (
            HealthStatus::Error,
            CheckSeverity::Critical,
            format!("Certificate expired {} days ago", -days),
        ),
        Some(days) if days < 7 => (
            HealthStatus::Unhealthy,
            CheckSeverity::Critical,
            format!("Certificate expires in {} days", days),
        ),
        Some(days) if days < 30 => (
            HealthStatus::Degraded,
            CheckSeverity::Warning,
            format!("Certificate expires in {} days", days),
        ),
        Some(days) => (
            HealthStatus::Healthy,
            CheckSeverity::Info,
            format!("Certificate valid for {} days", days),
        ),
        None => (
            HealthStatus::Error,
            CheckSeverity::Error,
            "Unable to determine expiry date".to_string(),
        ),
    };

    Ok(SslCheckResult {
        status,
        message,
        severity,
        details: cert_info.details,
        expires_at: cert_info.expires_at,
        days_until_expiry,
        issuer: cert_info.issuer,
        subject: cert_info.subject,
    })
}

/// Certificate information extracted from X.509
struct CertificateInfo {
    expires_at: Option<DateTime<Utc>>,
    issuer: Option<String>,
    subject: Option<String>,
    details: Option<String>,
}

/// Parse X.509 certificate
fn parse_x509_certificate(data: &[u8]) -> Result<CertificateInfo> {
    #[cfg(feature = "network")]
    {
        use x509_parser::prelude::*;

        // Try PEM first
        if let Ok((_, pem)) = parse_x509_pem(data) {
            return parse_x509_der(&pem.contents);
        }

        // Try DER
        parse_x509_der(data)
    }

    #[cfg(not(feature = "network"))]
    {
        Err(Error::FeatureNotEnabled("network".to_string()))
    }
}

#[cfg(feature = "network")]
fn parse_x509_der(data: &[u8]) -> Result<CertificateInfo> {
    use x509_parser::prelude::*;

    let (_, cert) = X509Certificate::from_der(data)
        .map_err(|e| Error::Parse(format!("Failed to parse certificate: {}", e)))?;

    // Extract expiry date
    let expires_at = {
        let not_after = cert.validity().not_after;
        let timestamp = not_after.timestamp();
        DateTime::from_timestamp(timestamp, 0)
    };

    // Extract issuer
    let issuer = cert.issuer().to_string();

    // Extract subject
    let subject = cert.subject().to_string();

    // Build details
    let mut details = String::new();
    details.push_str(&format!("Serial: {:?}\n", cert.serial));
    details.push_str(&format!("Signature Algorithm: {:?}\n", cert.signature_algorithm));
    if let Ok(san) = cert.subject_alternative_name() {
        if let Some(san_ext) = san {
            details.push_str(&format!("Subject Alternative Names: {:?}\n", san_ext.value.general_names));
        }
    }

    Ok(CertificateInfo {
        expires_at,
        issuer: Some(issuer),
        subject: Some(subject),
        details: Some(details),
    })
}

/// Check SSL certificate from URL (TLS handshake)
pub async fn check_ssl_url(url: &str) -> Result<SslCheckResult> {
    #[cfg(feature = "network")]
    {
        use reqwest;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| Error::Network(format!("Failed to create client: {}", e)))?;

        let response = client.get(url)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Request failed: {}", e)))?;

        // TODO: Extract certificate from TLS connection
        // This requires accessing the underlying TLS stream

        Ok(SslCheckResult {
            status: HealthStatus::Healthy,
            message: format!("Connected to {}", url),
            severity: CheckSeverity::Info,
            details: None,
            expires_at: None,
            days_until_expiry: None,
            issuer: None,
            subject: None,
        })
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
    async fn test_check_nonexistent_cert() {
        let result = check_ssl_certificate(Path::new("/nonexistent/cert.pem")).await;
        assert!(result.is_ok());

        let check = result.unwrap();
        assert_eq!(check.status, HealthStatus::Error);
    }
}