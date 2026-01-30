//! Example demonstrating network health checks
//!
//! Run with:
//! cargo run --example network_checks --features network





#[cfg(not(feature = "network"))]
fn main() {
    eprintln!("❌ This example requires the 'network' feature");
    eprintln!("Run with:");
    eprintln!("  cargo run --example network_checks --features network");
    std::process::exit(1);
}

#[cfg(feature = "network")]
#[tokio::main]
async fn main() -> nginx_discovery::Result<()> {
    use nginx_discovery::{
        network::{
            check_all, check_port, check_ssl_certificate, resolve_hostname, HealthStatus,
            NetworkCheckOptions,
        },
        parse,
    };
    use std::path::Path;

    println!("=== Network Check Examples ===\n");

    // 1. Port check
    println!("1. Port Check");
    if let Ok(result) = check_port("127.0.0.1", 80).await {
        println!("Status: {}", result.status);
        println!("Message: {}", result.message);
    }
    println!();

    // 2. DNS resolution
    println!("2. DNS Resolution");
    if let Ok(result) = resolve_hostname("example.com").await {
        println!("Status: {}", result.status);
        println!("Addresses: {:?}", result.addresses);
    }
    println!();

    // 3. SSL certificate check
    println!("3. SSL Certificate Check");
    if let Ok(result) = check_ssl_certificate(Path::new("/etc/ssl/certs/example.pem")).await {
        println!("Status: {}", result.status);
        println!("Message: {}", result.message);
    }
    println!();

    // 4. Full configuration-driven check
    println!("4. Full Configuration Check");

    let config_text = r#"
http {
    upstream backend {
        server 127.0.0.1 8080;
    }

    server {
        listen 80;
        server_name example.com;

        location / {
            proxy_pass http://backend;
        }
    }
}
"#;

    let config = parse(config_text)?;

    let options = NetworkCheckOptions {
        check_upstreams: true,
        check_ports: true,
        check_dns: true,
        check_ssl: false,
        ..Default::default()
    };

    let results = check_all(&config, options).await?;

    let healthy = results
        .iter()
        .filter(|r| r.status == HealthStatus::Healthy)
        .count();

    println!("Checks passed: {}/{}", healthy, results.len());

    for r in results {
        println!("- [{}] {} → {}", r.check_type, r.target, r.status);
    }

    println!("\n=== Done ===");
    Ok(())
}
