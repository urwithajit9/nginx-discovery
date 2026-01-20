# nginx-discovery

[![Crates.io](https://img.shields.io/crates/v/nginx-discovery.svg)](https://crates.io/crates/nginx-discovery)
[![Documentation](https://docs.rs/nginx-discovery/badge.svg)](https://docs.rs/nginx-discovery)
[![License](https://img.shields.io/crates/l/nginx-discovery.svg)](https://github.com/urwithajit9/nginx-discovery#license)
[![Build Status](https://github.com/urwithajit9/nginx-discovery/workflows/CI/badge.svg)](https://github.com/urwithajit9/nginx-discovery/actions)

**Discover and introspect NGINX configurations with ease.**

`nginx-discovery` is a Rust library and CLI tool for parsing, analyzing, and extracting information from NGINX configurations. Whether you're setting up log analysis, auditing security configurations, or automating operational tasks, this crate provides both high-level convenience APIs and low-level parsing capabilities.

## Features

- 🔍 **Auto-discovery**: Automatically detect and parse running NGINX instances
- 📊 **High-level API**: Simple methods to extract logs, servers, upstreams, and more
- 🎯 **Type-safe**: Strongly-typed Rust structs for all NGINX directives
- ⚡ **Fast**: Efficient parsing with detailed error messages
- 🔧 **Flexible**: Three API levels (high, mid, low) for different use cases
- 📦 **Zero-copy parsing**: Minimal allocations for performance
- 🛠️ **CLI tool**: Command-line interface for quick inspections

## Quick Start

### Library Usage

```rust
use nginx_discovery::prelude::*;

// Discover from running NGINX instance
let discovery = NginxDiscovery::from_running_instance()?;

// Get access logs
for log in discovery.access_logs() {
    println!("Log: {} (format: {})",
        log.path.display(),
        log.format_name.as_deref().unwrap_or("combined")
    );
}

// Get all server names
for name in discovery.server_names() {
    println!("Server: {}", name);
}

// Export to JSON
let json = discovery.to_json()?;
```

### CLI Usage

```bash
# Install the CLI tool
cargo install nginx-discovery --features cli

# Discover running NGINX
nginx-discover

# Analyze a specific config file
nginx-discover --config /etc/nginx/nginx.conf

# Export to JSON
nginx-discover --output json > nginx-config.json

# Extract only access logs
nginx-discover --extract logs
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
nginx-discovery = "0.1"
```

### Feature Flags

```toml
[dependencies]
nginx-discovery = { version = "0.1", features = ["serde", "system"] }
```

Available features:
- `system` (default): System interaction (detect nginx, run nginx -T)
- `serde`: JSON/YAML serialization support
- `visitor`: Visitor pattern for AST traversal
- `includes`: Include directive resolution
- `cli`: Command-line interface

## API Levels

### High-Level API (Discovery)

Perfect for most use cases:

```rust
let discovery = NginxDiscovery::from_config_file("/etc/nginx/nginx.conf")?;
let logs = discovery.access_logs();
let upstreams = discovery.upstreams();
```

### Mid-Level API (Extractors)

When you need more control:

```rust
use nginx_discovery::{parse, extract};

let config = parse(text)?;
let logs = extract::access_logs(&config)?;
let servers = extract::servers(&config)?
    .into_iter()
    .filter(|s| s.has_ssl())
    .collect::<Vec<_>>();
```

### Low-Level API (AST)

For custom processing:

```rust
use nginx_discovery::ast::*;

let config = nginx_discovery::parse(text)?;
for directive in &config.directives {
    // Custom logic here
}
```

## Examples

### Extract All Log Files

```rust
use nginx_discovery::prelude::*;

let discovery = NginxDiscovery::from_running_instance()?;
let log_files = discovery.all_log_files();

for path in log_files {
    println!("{}", path.display());
}
```

### Find SSL-Enabled Servers

```rust
use nginx_discovery::prelude::*;

let discovery = NginxDiscovery::from_running_instance()?;
let ssl_servers = discovery.servers()
    .into_iter()
    .filter(|s| s.ssl.is_some())
    .collect::<Vec<_>>();

for server in ssl_servers {
    println!("SSL Server: {:?}", server.server_names);
}
```

### Generate Monitoring Config

```rust
use nginx_discovery::prelude::*;

let discovery = NginxDiscovery::from_running_instance()?;

// Generate config for log monitoring tool
for log in discovery.access_logs() {
    println!("- path: {}", log.path.display());
    if let Some(format) = &log.format {
        println!("  pattern: {}", format.pattern);
    }
}
```

## Documentation

- [API Documentation](https://docs.rs/nginx-discovery)
- [Examples](https://github.com/urwithajit9/nginx-discovery/tree/main/examples)
- [Design Decisions](https://github.com/urwithajit9/nginx-discovery/blob/main/docs/design.md)
- [Contributing Guide](https://github.com/urwithajit9/nginx-discovery/blob/main/CONTRIBUTING.md)

## Use Cases

- **Log Analysis**: Discover log files and formats for tools like Fluentd, Vector, or Logstash
- **Monitoring Setup**: Extract upstreams and servers for monitoring systems
- **Security Auditing**: Find SSL configurations, authentication settings
- **Migration Planning**: Document current configuration state
- **Automated Operations**: Make decisions based on current NGINX setup

## Supported NGINX Directives (v0.1.0)

- ✅ `log_format` - Log format definitions
- ✅ `access_log` - Access log configurations
- ✅ `error_log` - Error log configurations
- ✅ `server` - Server blocks
- ✅ `location` - Location blocks
- ✅ Basic directives (listen, server_name, root, etc.)

## Roadmap

- [ ] Upstream block extraction (v0.2.0)
- [ ] SSL/TLS configuration extraction (v0.2.0)
- [ ] Include directive resolution (v0.3.0)
- [ ] Map directive support (v0.3.0)
- [ ] Variable resolution (v0.4.0)
- [ ] Conditional evaluation (v0.4.0)
- [ ] Config validation (v0.5.0)

## Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) first.

### Development

```bash
# Clone the repository
git clone https://github.com/urwithajit9/nginx-discovery.git
cd nginx-discovery

# Run tests
cargo test

# Run with all features
cargo test --all-features

# Run benchmarks
cargo bench

# Build documentation
cargo doc --all-features --open

# Format code
cargo fmt

# Lint
cargo clippy --all-features -- -D warnings
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Acknowledgments

Built with ❤️ in Rust. Inspired by the need for better NGINX configuration tooling in the Rust ecosystem.

---

**Author**: Ajit Kumar ([@urwithajit9](https://github.com/urwithajit9))