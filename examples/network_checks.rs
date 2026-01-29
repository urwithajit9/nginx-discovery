// examples/network_checks.rs
//! Example demonstrating network health checks

#![cfg(feature = "network")]

use nginx_discovery::{
    parse,
    network::{
        check_port, check_ssl_certificate, resolve_hostname, check_all,
        NetworkCheckOptions, UpstreamBackend, check_upstream,
        check_upstream_http, HealthStatus,
    },
};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Network Check Examples ===\n");

    // Example 1: Check port availability
    println!("1. Port Check:");
    println!("--------------");
    match check_port("127.0.0.1", 80).await {
        Ok(result) => {
            println!("Status: {}", result.status);
            println!("Message: {}", result.message);
            if result.is_listening {
                println!("✓ Port is listening");
            } else {
                println!("✗ Port is not listening");
            }
            if let Some(latency) = result.latency {
                println!("Latency: {:?}", latency);
            }
        }
        Err(e) => println!("Error: {}", e),
    }
    println!();

    // Example 2: DNS resolution
    println!("2. DNS Resolution:");
    println!("------------------");
    match resolve_hostname("example.com").await {
        Ok(result) => {
            println!("Status: {}", result.status);
            println!("Message: {}", result.message);
            println!("Addresses: {:?}", result.addresses);
            if let Some(time) = result.resolution_time {
                println!("Resolution time: {:?}", time);
            }
        }
        Err(e) => println!("Error: {}", e),
    }
    println!();

    // Example 3: SSL certificate check
    println!("3. SSL Certificate Check:");
    println!("-------------------------");
    // Using a hypothetical certificate path
    match check_ssl_certificate(Path::new("/etc/ssl/certs/example.pem")).await {
        Ok(result) => {
            println!("Status: {}", result.status);
            println!("Message: {}", result.message);
            if let Some(expires) = result.expires_at {
                println!("Expires: {}", expires);
            }
            if let Some(days) = result.days_until_expiry {
                println!("Days until expiry: {}", days);
            }
            if let Some(issuer) = &result.issuer {
                println!("Issuer: {}", issuer);
            }
        }
        Err(e) => println!("Error: {}", e),
    }
    println!();

    // Example 4: Upstream backend check
    println!("4. Upstream Backend Check:");
    println!("--------------------------");
    let backend = UpstreamBackend {
        host: "backend1.example.com".to_string(),
        port: 8080,
        weight: Some(1),
        max_fails: Some(3),
        fail_timeout: None,
    };

    match check_upstream(&backend).await {
        Ok(result) => {
            println!("Status: {}", result.status);
            println!("Message: {}", result.message);
            if let Some(latency) = result.latency {
                println!("Latency: {:?}", latency);
            }
        }
        Err(e) => println!("Error: {}", e),
    }
    println!();

    // Example 5: HTTP health check
    println!("5. HTTP Health Check:");
    println!("---------------------");
    let backend = UpstreamBackend {
        host: "httpbin.org".to_string(),
        port: 80,
        weight: Some(1),
        max_fails: Some(3),
        fail_timeout: None,
    };

    match check_upstream_http(&backend, "/status/200").await {
        Ok(result) => {
            println!("Status: {}", result.status);
            println!("Message: {}", result.message);
            if let Some(details) = &result.details {
                println!("Details: {}", details);
            }
        }
        Err(e) => println!("Error: {}", e),
    }
    println!();

    // Example 6: Comprehensive check of NGINX configuration
    println!("6. Full Configuration Check:");
    println!("----------------------------");
    let config_text = r#"
http {
    server {
        listen 80;
        listen 443 ssl;
        server_name example.com;

        ssl_certificate /etc/ssl/cert.pem;
        ssl_certificate_key /etc/ssl/key.pem;
    }
}
"#;

    let config = parse(config_text)?;

    // Configure which checks to run
    let options = NetworkCheckOptions {
        check_upstreams: false,
        check_ssl: true,
        check_ports: true,
        check_dns: true,
        ..Default::default()
    };

    match check_all(&config, options).await {
        Ok(results) => {
            println!("Total checks: {}", results.len());

            for result in &results {
                println!("\n  Check: {}", result.check_type);
                println!("  Target: {}", result.target);
                println!("  Status: {}", result.status);
                println!("  Severity: {}", result.severity);
                println!("  Message: {}", result.message);

                if let Some(details) = &result.details {
                    println!("  Details: {}", details);
                }
            }

            // Summary
            let healthy = results.iter()
                .filter(|r| r.status == HealthStatus::Healthy)
                .count();
            let total = results.len();

            println!("\n  Summary: {}/{} checks passed", healthy, total);
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\n=== Network Check Examples Complete ===");

    Ok(())
}