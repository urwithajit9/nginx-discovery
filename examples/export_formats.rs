// examples/export_formats.rs
//! Example demonstrating different export formats

use nginx_discovery::{
    parse,
    export::{export, ExportOptions, ExportFormat, Filter, FilterType},
};
use std::io;

fn main() -> anyhow::Result<()> {
    // Sample NGINX configuration
    let config_text = r#"
http {
    server {
        listen 80;
        listen 443 ssl;
        server_name example.com www.example.com;
        root /var/www/html;

        ssl_certificate /etc/ssl/cert.pem;
        ssl_certificate_key /etc/ssl/key.pem;

        location / {
            proxy_pass http://backend;
            proxy_set_header Host $host;
        }

        location /api {
            proxy_pass http://api_backend;
        }
    }

    server {
        listen 8080;
        server_name api.example.com;

        location / {
            return 200 "API Server";
        }
    }
}
"#;

    // Parse configuration
    let config = parse(config_text)?;

    println!("=== Export Examples ===\n");

    // Example 1: Export to JSON (pretty)
    println!("1. JSON Export (pretty):");
    println!("-----------------------");
    let json_options = ExportOptions::builder()
        .format(ExportFormat::Json)
        .pretty(true)
        .include_metadata(true)
        .build();

    export(&config, &mut io::stdout(), json_options)?;
    println!("\n");

    // Example 2: Export to YAML
    println!("2. YAML Export:");
    println!("---------------");
    let yaml_options = ExportOptions::builder()
        .format(ExportFormat::Yaml)
        .build();

    export(&config, &mut io::stdout(), yaml_options)?;
    println!("\n");

    // Example 3: Export to TOML (if feature enabled)
    #[cfg(feature = "export-toml")]
    {
        println!("3. TOML Export:");
        println!("---------------");
        let toml_options = ExportOptions::builder()
            .format(ExportFormat::Toml)
            .pretty(true)
            .build();

        export(&config, &mut io::stdout(), toml_options)?;
        println!("\n");
    }

    // Example 4: Export to Markdown (if feature enabled)
    #[cfg(feature = "export-markdown")]
    {
        println!("4. Markdown Export:");
        println!("-------------------");
        let md_options = ExportOptions::builder()
            .format(ExportFormat::Markdown)
            .include_metadata(true)
            .build();

        export(&config, &mut io::stdout(), md_options)?;
        println!("\n");
    }

    // Example 5: Export with filtering
    println!("5. Filtered Export (server_name=example.com):");
    println!("---------------------------------------------");
    let filter = Filter::new(FilterType::ServerName, "example.com");
    let filtered_options = ExportOptions::builder()
        .format(ExportFormat::Json)
        .pretty(true)
        .filter(filter)
        .build();

    export(&config, &mut io::stdout(), filtered_options)?;
    println!("\n");

    // Example 6: Compact export
    println!("6. Compact JSON Export:");
    println!("-----------------------");
    let compact_options = ExportOptions::builder()
        .format(ExportFormat::Json)
        .pretty(false)
        .compact(true)
        .build();

    export(&config, &mut io::stdout(), compact_options)?;
    println!("\n");

    println!("=== Export Examples Complete ===");

    Ok(())
}