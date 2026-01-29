// examples/network_checks.rs
//! Example demonstrating network health checks
//!
//! Run with: cargo run --example network_checks --features network

#[cfg(not(feature = "network"))]
fn main() {
    eprintln!("❌ This example requires the 'network' feature");
    eprintln!();
    eprintln!("Run with:");
    eprintln!("  cargo run --example network_checks --features network");
    eprintln!();
    eprintln!("Or add to Cargo.toml:");
    eprintln!("  nginx-discovery = {{ version = \"0.4.0\", features = [\"network\"] }}");
    std::process::exit(1);
}

#[cfg(feature = "network")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use nginx_discovery::{
        parse,
        network::{
            check_port, check_ssl_certificate, resolve_hostname, check_all,
            NetworkCheckOptions, UpstreamBackend, check_upstream,
            check_upstream_http, HealthStatus,
        },
    };
    use std::path::Path;

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
        host: "localhost".to_string(),
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
        check_ssl: false,  // Would need real cert file
        check_ports: false,
        check_dns: true,   // Can check example.com
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