//! Extract server blocks from NGINX configuration
//!
//! This example demonstrates how to extract and analyze server blocks,
//! including listen directives, locations, and SSL configuration.

use nginx_discovery::NginxDiscovery;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = r#"
    http {
        server {
            listen 80;
            listen 443 ssl http2;
            server_name example.com www.example.com;
            root /var/www/example;

            location / {
                try_files $uri $uri/ =404;
            }

            location /api {
                proxy_pass http://backend:3000;
            }

            location ~ \.php$ {
                fastcgi_pass unix:/var/run/php-fpm.sock;
            }
        }

        server {
            listen 8080;
            server_name api.example.com;

            location / {
                proxy_pass http://api-backend:8080;
            }
        }
    }
    "#;

    let discovery = NginxDiscovery::from_config_text(config)?;

    // Extract all servers
    println!("=== All Servers ===");
    let servers = discovery.servers();
    println!("Found {} server blocks\n", servers.len());

    for (i, server) in servers.iter().enumerate() {
        println!("Server {}:", i + 1);

        // Server names
        if !server.server_names.is_empty() {
            println!("  Names: {}", server.server_names.join(", "));
        }

        // Listen directives
        println!("  Listening on:");
        for listen in &server.listen {
            let mut options = vec![];
            if listen.ssl {
                options.push("ssl");
            }
            if listen.http2 {
                options.push("http2");
            }
            if listen.http3 {
                options.push("http3");
            }
            if listen.default_server {
                options.push("default_server");
            }

            let opts = if options.is_empty() {
                String::new()
            } else {
                format!(" ({})", options.join(", "))
            };

            println!("    {}:{}{}", listen.address, listen.port, opts);
        }

        // Root directory
        if let Some(root) = &server.root {
            println!("  Root: {}", root.display());
        }

        // Locations
        if !server.locations.is_empty() {
            println!("  Locations:");
            for location in &server.locations {
                let modifier = match location.modifier {
                    nginx_discovery::types::LocationModifier::None => "",
                    nginx_discovery::types::LocationModifier::Exact => "= ",
                    nginx_discovery::types::LocationModifier::PrefixPriority => "^~ ",
                    nginx_discovery::types::LocationModifier::Regex => "~ ",
                    nginx_discovery::types::LocationModifier::RegexCaseInsensitive => "~* ",
                };

                let loc_type = if location.is_proxy() {
                    format!("(proxy -> {})", location.proxy_pass.as_ref().unwrap())
                } else if location.is_static() {
                    format!("(static: {})", location.root.as_ref().unwrap().display())
                } else {
                    "(other)".to_string()
                };

                println!("    {}{} {}", modifier, location.path, loc_type);
            }
        }

        println!();
    }

    // SSL servers
    println!("=== SSL Servers ===");
    let ssl_servers = discovery.ssl_servers();
    println!("Found {} SSL-enabled servers", ssl_servers.len());
    for server in ssl_servers {
        if let Some(name) = server.primary_name() {
            println!("  - {}", name);
        }
    }
    println!();

    // Listening ports
    println!("=== Listening Ports ===");
    let ports = discovery.listening_ports();
    println!("Ports: {:?}", ports);
    println!();

    // Proxy locations
    println!("=== Proxy Locations ===");
    let proxies = discovery.proxy_locations();
    println!("Found {} proxy locations", proxies.len());
    for proxy in proxies {
        println!("  {} -> {}", proxy.path, proxy.proxy_pass.as_ref().unwrap());
    }

    Ok(())
}
