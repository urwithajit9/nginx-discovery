# nginx-discovery

Parse, analyze, and extract information from NGINX configurations with powerful export and health checking capabilities.

[![Crates.io](https://img.shields.io/crates/v/nginx-discovery.svg)](https://crates.io/crates/nginx-discovery)
[![Documentation](https://docs.rs/nginx-discovery/badge.svg)](https://docs.rs/nginx-discovery)
[![License](https://img.shields.io/crates/l/nginx-discovery.svg)](LICENSE)
[![Build Status](https://github.com/urwithajit9/nginx-discovery/workflows/CI/badge.svg)](https://github.com/urwithajit9/nginx-discovery/actions)

## Features

- 🔍 **Parse NGINX configs** - Parse any NGINX configuration file
- 📊 **Multiple Export Formats** - Export to JSON, YAML, TOML, or Markdown
- 🔌 **Export Filtering** - Filter by server name, port, SSL status, or directive
- 🌐 **Network Health Checks** - Check port availability, DNS resolution, SSL certificates, and upstream backends
- 🐚 **Shell Completions** - Generate completions for Bash, Zsh, Fish, PowerShell, Elvish
- 🚀 **High Performance** - Fast parsing with minimal memory footprint
- 📦 **Library & CLI** - Use as a library or standalone CLI tool

## Installation

### As a Library

Add to your `Cargo.toml`:
```toml
[dependencies]
nginx-discovery = "0.4.0"

# With optional features
nginx-discovery = { version = "0.4.0", features = ["serde", "network", "export-all"] }
```

### As a CLI Tool
```bash
cargo install nginx-discovery --features cli
```

## Quick Start

### Library Usage
```rust
use nginx_discovery::{parse, extract::servers};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse configuration
    let config = parse(r#"
        http {
            server {
                listen 80;
                server_name example.com;
                location / {
                    proxy_pass http://backend;
                }
            }
        }
    "#)?;

    // Extract servers
    let servers = servers(&config)?;
    println!("Found {} servers", servers.len());

    Ok(())
}
```

### Export to Different Formats
```rust
use nginx_discovery::{parse, export::{export, ExportOptions, ExportFormat}};
use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = parse(nginx_config)?;

    // Export to JSON
    let options = ExportOptions::builder()
        .format(ExportFormat::Json)
        .pretty(true)
        .build();

    export(&config, &mut io::stdout(), options)?;
    Ok(())
}
```

### Network Health Checks
```rust
use nginx_discovery::network::{check_port, resolve_hostname};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if port is available
    let result = check_port("127.0.0.1", 80).await?;
    println!("Port status: {}", result.status);

    // Resolve hostname
    let result = resolve_hostname("example.com").await?;
    println!("Addresses: {:?}", result.addresses);

    Ok(())
}
```

### CLI Usage
```bash
# Parse and display configuration
nginx-discover parse /etc/nginx/nginx.conf

# Export to JSON
nginx-discover export --format json output.json

# Export to Markdown report
nginx-discover export --format markdown --features export-markdown report.md

# Filter by server name
nginx-discover export --filter "server_name=*.example.com" output.json

# Check network health
nginx-discover network check-all

# Check specific ports
nginx-discover network check-ports

# Generate shell completions
nginx-discover completions bash > ~/.bash_completion.d/nginx-discover
```

## Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `serde` | Enable serialization support (JSON, YAML) | ✅ |
| `system` | Enable system interaction (file finding) | ✅ |
| `export-toml` | Enable TOML export format | ❌ |
| `export-markdown` | Enable Markdown export format | ❌ |
| `export-all` | Enable all export formats | ❌ |
| `network` | Enable network health checking | ❌ |
| `cli` | Enable CLI binary | ❌ |
| `full` | Enable all features | ❌ |

## Examples

See the [examples/](examples/) directory for more examples:

- [export_formats.rs](examples/export_formats.rs) - Export to different formats
- [network_checks.rs](examples/network_checks.rs) - Network health checking

## Documentation

- [API Documentation](https://docs.rs/nginx-discovery)
- [Changelog](CHANGELOG.md)
- [Contributing Guide](CONTRIBUTING.md)

## Requirements

- Rust 1.70.0 or later
- For network features: tokio runtime

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## Acknowledgments

Built with ❤️ for the NGINX community.