// tests/network_integration.rs
//! Integration tests for network checking functionality

#![cfg(feature = "network")]

use nginx_discovery::{
    parse,
    network::{
        check_port, check_ssl_certificate, resolve_hostname, check_all,
        NetworkCheckOptions, HealthStatus, UpstreamBackend, check_upstream,
    },
};
use std::path::Path;

const SAMPLE_CONFIG: &str = r#"
http {
    server {
        listen 80;
        listen 443 ssl;
        server_name example.com;

        ssl_certificate /etc/ssl/cert.pem;
        ssl_certificate_key /etc/ssl/key.pem;

        location / {
            proxy_pass http://backend;
        }
    }
}
"#;

#[tokio::test]
async fn test_check_port_localhost() {
    // Test checking localhost on a port that likely doesn't exist
    let result = check_port("127.0.0.1", 59999).await;
    assert!(result.is_ok());

    let check = result.unwrap();
    assert_eq!(check.port, 59999);
    assert_eq!(check.address, "127.0.0.1");
}

#[tokio::test]
async fn test_resolve_localhost() {
    let result = resolve_hostname("localhost").await;
    assert!(result.is_ok());

    let check = result.unwrap();
    assert_eq!(check.status, HealthStatus::Healthy);
    assert!(!check.addresses.is_empty());
    assert_eq!(check.hostname, "localhost");
}

#[tokio::test]
async fn test_resolve_invalid_hostname() {
    let result = resolve_hostname("this-domain-does-not-exist-12345.invalid").await;
    assert!(result.is_ok());

    let check = result.unwrap();
    // Should either be Error or Unhealthy
    assert!(matches!(check.status, HealthStatus::Error | HealthStatus::Unhealthy));
}

#[tokio::test]
async fn test_check_ssl_nonexistent() {
    let result = check_ssl_certificate(Path::new("/nonexistent/cert.pem")).await;
    assert!(result.is_ok());

    let check = result.unwrap();
    assert_eq!(check.status, HealthStatus::Error);
}

#[tokio::test]
async fn test_check_upstream() {
    let backend = UpstreamBackend {
        host: "127.0.0.1".to_string(),
        port: 59999, // Unlikely to be in use
        weight: Some(1),
        max_fails: Some(3),
        fail_timeout: None,
    };

    let result = check_upstream(&backend).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_network_check_options() {
    let options = NetworkCheckOptions::default();
    assert!(options.check_upstreams);
    assert!(options.check_ssl);
    assert!(options.check_ports);
    assert!(options.check_dns);
}

#[tokio::test]
async fn test_network_check_options_presets() {
    let ssl_only = NetworkCheckOptions::ssl_only();
    assert!(!ssl_only.check_upstreams);
    assert!(ssl_only.check_ssl);
    assert!(!ssl_only.check_ports);
    assert!(!ssl_only.check_dns);

    let ports_only = NetworkCheckOptions::ports_only();
    assert!(!ports_only.check_upstreams);
    assert!(!ports_only.check_ssl);
    assert!(ports_only.check_ports);
    assert!(!ports_only.check_dns);
}

#[tokio::test]
async fn test_check_all_with_config() {
    let config = parse(SAMPLE_CONFIG).expect("Failed to parse config");

    let options = NetworkCheckOptions {
        check_upstreams: false, // Skip upstreams for now
        check_ssl: false,       // Would need real cert file
        check_ports: false,     // Would need open ports
        check_dns: true,        // Can check example.com
        ..Default::default()
    };

    let result = check_all(&config, options).await;
    assert!(result.is_ok());
}

#[test]
fn test_health_status_display() {
    assert_eq!(HealthStatus::Healthy.to_string(), "HEALTHY");
    assert_eq!(HealthStatus::Unhealthy.to_string(), "UNHEALTHY");
    assert_eq!(HealthStatus::Degraded.to_string(), "DEGRADED");
    assert_eq!(HealthStatus::Error.to_string(), "ERROR");
}

#[tokio::test]
async fn test_multiple_dns_resolutions() {
    use nginx_discovery::network::resolve_hostnames;

    let hostnames = vec![
        "localhost".to_string(),
        "invalid-hostname-12345.invalid".to_string(),
    ];

    let results = resolve_hostnames(hostnames).await;
    assert!(results.is_ok());

    let checks = results.unwrap();
    assert_eq!(checks.len(), 2);
}