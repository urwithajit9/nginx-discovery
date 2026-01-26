# nginx-discovery

[![Crates.io](https://img.shields.io/crates/v/nginx-discovery.svg)](https://crates.io/crates/nginx-discovery)
[![Documentation](https://docs.rs/nginx-discovery/badge.svg)](https://docs.rs/nginx-discovery)
[![License](https://img.shields.io/crates/l/nginx-discovery.svg)](https://github.com/urwithajit9/nginx-discovery#license)
[![Build Status](https://github.com/urwithajit9/nginx-discovery/workflows/CI/badge.svg)](https://github.com/urwithajit9/nginx-discovery/actions)

**Discover and parse NGINX configurations with ease.**

A Rust library for parsing, analyzing, and extracting information from NGINX configuration files. Perfect for building tools that need to understand NGINX configs programmatically.

## ✨ Features

- 🔍 **Parse NGINX Configs** - Full support for directives, blocks, and nested structures
- 🖥️ **Server Block Extraction** - Extract and analyze server blocks with listen directives and locations
- 📊 **Extract Information** - High-level extractors for logs, servers, locations, and more
- 🎯 **Type-Safe** - Strongly-typed AST and configuration objects
- ⚡ **Fast** - Efficient lexer and parser with zero-copy where possible
- 🛠️ **Great Errors** - Detailed error messages with source locations and suggestions
- 📚 **Well Documented** - Comprehensive docs and examples

## 🚀 Quick Start

Add to your `Cargo.toml`:
```toml
[dependencies]
nginx-discovery = "0.2"
```

### Parse a Configuration
```rust
use nginx_discovery::parse;

let config = r#"
server {
    listen 80;
    server_name example.com;

    location / {
        root /var/www/html;
    }
}
"#;

let parsed = parse(config)?;
println!("Found {} directives", parsed.directives.len());
```

### Extract Servers (New in v0.2.0)
```rust
use nginx_discovery::NginxDiscovery;

let config = r#"
server {
    listen 80;
    listen 443 ssl;
    server_name example.com www.example.com;

    location / {
        root /var/www/html;
    }

    location /api {
        proxy_pass http://backend:3000;
    }
}
"#;

let discovery = NginxDiscovery::from_config_text(config)?;

// Get all servers
let servers = discovery.servers();
println!("Found {} servers", servers.len());

// Get SSL servers only
let ssl_servers = discovery.ssl_servers();
println!("SSL servers: {}", ssl_servers.len());

// Get all listening ports
let ports = discovery.listening_ports();
println!("Listening on ports: {:?}", ports);

// Get proxy locations
let proxies = discovery.proxy_locations();
for location in proxies {
    println!("Proxy: {} -> {:?}", location.path, location.proxy_pass);
}
```

### Extract Access Logs
```rust
use nginx_discovery::{parse, extract};

let config = parse(config_text)?;
let logs = extract::access_logs(&config)?;

for log in logs {
    println!("Log: {}", log.path.display());
    println!("Format: {:?}", log.format_name);
    println!("Context: {:?}", log.context);
}
```

### Extract Log Formats
```rust
use nginx_discovery::{parse, extract};

let config = parse(config_text)?;
let formats = extract::log_formats(&config)?;

for format in formats {
    println!("Format: {}", format.name());
    println!("Variables: {:?}", format.variables());
}
```

## 📖 Examples

Check out the [`examples/`](examples/) directory:

- [`extract_servers.rs`](examples/extract_servers.rs) - Extract server blocks and analyze configurations
- [`extract_logs.rs`](examples/extract_logs.rs) - Extract log configurations
- [`parse_config.rs`](examples/parse_config.rs) - Parse NGINX configuration
- [`lex_config.rs`](examples/lex_config.rs) - Tokenize NGINX configs
- [`test_nginx_detection.rs`](examples/test_nginx_detection.rs) - Test NGINX system detection (requires `system` feature)

Run an example:
```bash
cargo run --example extract_servers
```

Run the system detection example:
```bash
cargo run --example test_nginx_detection --features system
```

## 🎯 Use Cases

- **Log Analysis Tools** - Extract log paths and formats for Fluentd, Vector, Logstash
- **Configuration Management** - Validate and analyze NGINX configs
- **Monitoring Setup** - Discover upstreams and servers for monitoring
- **Migration Tools** - Parse existing configs for migration planning
- **Documentation** - Auto-generate documentation from configs
- **Security Auditing** - Analyze SSL/TLS and security settings
- **Service Discovery** - Extract server names, ports, and proxy configurations

## 🏗️ Architecture

The library is organized in layers:
```
┌─────────────────────────────────────┐
│   High-Level API (NginxDiscovery)  │  ← Convenient discovery methods
├─────────────────────────────────────┤
│   Extractors (extract::*)          │  ← Extract servers, logs, etc.
├─────────────────────────────────────┤
│   Parser (parse)                    │  ← Convert tokens to AST
├─────────────────────────────────────┤
│   Lexer (tokenize)                  │  ← Convert text to tokens
├─────────────────────────────────────┤
│   AST Types                         │  ← Type-safe representation
└─────────────────────────────────────┘
```

### Low-Level: Lexer
```rust
use nginx_discovery::parser::Lexer;

let mut lexer = Lexer::new("server { listen 80; }");
let tokens = lexer.tokenize()?;
```

### Mid-Level: Parser
```rust
use nginx_discovery::parse;

let config = parse("server { listen 80; }")?;
for directive in &config.directives {
    println!("{}", directive.name());
}
```

### High-Level: Extractors
```rust
use nginx_discovery::{parse, extract};

let config = parse(config_text)?;
let servers = extract::servers(&config)?;
let logs = extract::access_logs(&config)?;
```

### Highest-Level: Discovery API
```rust
use nginx_discovery::NginxDiscovery;

let discovery = NginxDiscovery::from_config_text(config)?;
let ssl_servers = discovery.ssl_servers();
let ports = discovery.listening_ports();
```

## 📚 Documentation

- [API Documentation](https://docs.rs/nginx-discovery)
- [Examples](examples/)
- [Contributing Guide](CONTRIBUTING.md)
- [Changelog](CHANGELOG.md)

## 🧪 Testing

The library has comprehensive test coverage:
```bash
# Run all tests
cargo test --all-features

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

## 🔧 Feature Flags
```toml
[dependencies]
nginx-discovery = { version = "0.2", features = ["serde"] }
```

Available features:

- `serde` - Serialize/deserialize AST types
- `system` (default) - System interaction utilities

## 📊 Supported NGINX Directives

Currently supports:

- ✅ Simple directives: `user nginx;`
- ✅ Block directives: `server { ... }`, `location { ... }`
- ✅ Nested blocks: `http { server { location { } } }`
- ✅ Quoted strings: `"value"` and `'value'`
- ✅ Variables: `$host`, `${variable}`
- ✅ Numbers: `80`, `443`, `1024`
- ✅ Comments: `# comment`
- ✅ Log formats and access logs
- ✅ **Server blocks** with listen directives and locations (v0.2.0)
- ✅ **Listen directives** with SSL, HTTP/2, HTTP/3 options (v0.2.0)
- ✅ **Location blocks** with all modifiers (=, ^~, ~, ~*) (v0.2.0)
- ✅ **Proxy detection** and static file serving (v0.2.0)

## 🗺️ Roadmap

### v0.2.0 (Released)
- ✅ Server block extractor
- ✅ Location block extractor
- ✅ SSL/TLS server detection
- ✅ Proxy location detection

### v0.3.0 (Planned)
- [ ] Upstream extractor
- [ ] Map directive support
- [ ] Geo/GeoIP support
- [ ] Include directive resolution
- [ ] Rate limiting configuration
- [ ] Auth configuration extractor

### v1.0.0 (Future)
- [ ] Complete NGINX directive support
- [ ] Configuration validation
- [ ] Config transformation tools

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details.

### Development
```bash
git clone https://github.com/urwithajit9/nginx-discovery.git
cd nginx-discovery

# Run tests
cargo test --all-features

# Run examples
cargo run --example extract_servers

# Format code
cargo fmt

# Run linter
cargo clippy --all-features -- -D warnings
```

## 📝 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## 🙏 Acknowledgments

Built with ❤️ in Rust.

Special thanks to the Rust community for excellent parser libraries and documentation.

## 📬 Contact

- **Author**: Ajit Kumar
- **GitHub**: [@urwithajit9](https://github.com/urwithajit9)
- **Issues**: [GitHub Issues](https://github.com/urwithajit9/nginx-discovery/issues)

---

**Star ⭐ this repo if you find it useful!**