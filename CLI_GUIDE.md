# nginx-discover CLI Guide

Complete guide for the `nginx-discover` command-line interface.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Global Options](#global-options)
- [Commands](#commands)
  - [parse](#parse---parse-and-validate-configuration)
  - [extract](#extract---extract-information)
  - [export](#export---export-configuration)
  - [doctor](#doctor---health-check)
- [Examples](#examples)
- [Output Formats](#output-formats)
- [Tips & Tricks](#tips--tricks)
- [Troubleshooting](#troubleshooting)

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/urwithajit9/nginx-discovery.git
cd nginx-discovery

# Build with CLI feature
cargo build --release --features cli

# Install globally
cargo install --path . --features cli

# Or copy to system path
sudo cp ./target/release/nginx-discover /usr/local/bin/
```

### Verify Installation

```bash
nginx-discover --version
nginx-discover --help
```

## Quick Start

```bash
# Run health check
sudo nginx-discover doctor

# Parse your NGINX configuration
sudo nginx-discover parse --tree

# Extract all servers
sudo nginx-discover extract servers

# Export to JSON
sudo nginx-discover export json --pretty
```

> **Note:** Many commands require `sudo` to access NGINX configuration files and test configuration syntax.

## Global Options

These options are available for all commands:

```
-c, --config <PATH>      Path to nginx.conf (auto-detected if not specified)
-v, --verbose            Enable verbose output
-q, --quiet              Suppress all output except errors
--color <WHEN>           When to use color (auto, always, never)
-h, --help               Show help
-V, --version            Show version
```

### Examples

```bash
# Use specific config file
nginx-discover --config /custom/path/nginx.conf parse

# Verbose output
nginx-discover --verbose extract servers

# No colors (for piping)
nginx-discover --color never extract logs

# Quiet mode
nginx-discover --quiet doctor
```

## Commands

### `parse` - Parse and Validate Configuration

Parse and display NGINX configuration structure.

#### Usage

```bash
nginx-discover parse [OPTIONS]
```

#### Options

```
-t, --tree       Display configuration as a tree
-s, --summary    Show summary only
    --json       Output as JSON
```

#### Examples

**Display tree and summary (default):**

```bash
sudo nginx-discover parse
```

Output:
```
Configuration Tree:

└─ user www-data;
└─ worker_processes auto;
└─ http {
  ├─ access_log /var/log/nginx/access.log;
  └─ server {
    ├─ listen 80;
    └─ server_name example.com;
    }
  }

=== Configuration Summary ===

  Total directives: 15
  Server blocks: 2
    - SSL enabled: 1
    - HTTP only: 1
  Listening ports: [80, 443]
  Access logs: 2
  Log formats: 1
  Location blocks: 5
    - Proxy locations: 2

✓ Configuration parsed successfully
```

**Tree view only:**

```bash
sudo nginx-discover parse --tree
```

**Summary only:**

```bash
sudo nginx-discover parse --summary
```

**Export AST as JSON:**

```bash
sudo nginx-discover parse --json > ast.json
```

---

### `extract` - Extract Information

Extract specific information from NGINX configuration.

#### Subcommands

- `servers` - Extract server blocks
- `logs` - Extract log configurations
- `locations` - Extract location blocks

---

#### `extract servers` - Extract Server Blocks

```bash
nginx-discover extract servers [OPTIONS]
```

**Options:**

```
    --ssl-only           Show only SSL-enabled servers
    --port <PORT>        Filter by port number
    --name <NAME>        Filter by server name (supports wildcards)
-f, --format <FORMAT>    Output format (table, json, yaml, csv) [default: table]
-o, --output <FILE>      Write to file instead of stdout
```

**Examples:**

```bash
# Show all servers in table format
sudo nginx-discover extract servers
```

Output:
```
╭─────────────────────────┬──────┬─────┬────────────┬───────────╮
│ Server Name             │ Port │ SSL │ Locations  │ Default   │
├─────────────────────────┼──────┼─────┼────────────┼───────────┤
│ example.com             │ 80   │ No  │ 3          │ No        │
│ api.example.com         │ 443  │ Yes │ 5          │ No        │
│ secure.example.com      │ 443  │ Yes │ 2          │ Yes       │
╰─────────────────────────┴──────┴─────┴────────────┴───────────╯

Total: 3 servers (2 with SSL)
```

**Show only SSL servers:**

```bash
sudo nginx-discover extract servers --ssl-only
```

**Filter by port:**

```bash
sudo nginx-discover extract servers --port 443
```

**Filter by name with wildcards:**

```bash
# All subdomains of example.com
sudo nginx-discover extract servers --name "*.example.com"

# Servers ending with .api.com
sudo nginx-discover extract servers --name "*.api.com"
```

**Export as JSON:**

```bash
sudo nginx-discover extract servers --format json --output servers.json
```

**Export as CSV:**

```bash
sudo nginx-discover extract servers --format csv > servers.csv
```

---

#### `extract logs` - Extract Log Configurations

```bash
nginx-discover extract logs [OPTIONS]
```

**Options:**

```
    --with-formats       Include log format definitions
    --context <CONTEXT>  Filter by context (http, server, location)
-f, --format <FORMAT>    Output format (table, json, yaml, csv) [default: table]
-o, --output <FILE>      Write to file
```

**Examples:**

```bash
# Show all access logs
sudo nginx-discover extract logs
```

Output:
```
╭──────────────────────────────────────┬──────────┬────────────────╮
│ Log File                             │ Format   │ Context        │
├──────────────────────────────────────┼──────────┼────────────────┤
│ /var/log/nginx/access.log            │ combined │ Http           │
│ /var/log/nginx/api-access.log        │ json     │ Server("api")  │
│ /var/log/nginx/app.log               │ main     │ Location("/")  │
╰──────────────────────────────────────┴──────────┴────────────────╯

Total: 3 log files
```

**Include format definitions:**

```bash
sudo nginx-discover extract logs --with-formats
```

Output includes:
```
Log Formats:

  main:
    Variables: remote_addr, remote_user, time_local, request, status

  json:
    Variables: remote_addr, time_local, request, status, body_bytes_sent
```

**Filter by context:**

```bash
# Only server-level logs
sudo nginx-discover extract logs --context server

# Only location-level logs
sudo nginx-discover extract logs --context location
```

**Export for log collection tool:**

```bash
# Generate JSON for Fluentd/Vector configuration
sudo nginx-discover extract logs --with-formats --format json -o logs-config.json
```

---

#### `extract locations` - Extract Location Blocks

```bash
nginx-discover extract locations [OPTIONS]
```

**Options:**

```
    --proxy-only         Show only proxy locations
    --static-only        Show only static file locations
    --server <NAME>      Filter by server name
-f, --format <FORMAT>    Output format (table, json, yaml, csv) [default: table]
-o, --output <FILE>      Write to file
```

**Examples:**

```bash
# Show all locations
sudo nginx-discover extract locations
```

Output:
```
╭─────────────────┬──────────┬──────────┬────────┬──────────────────────────╮
│ Server          │ Path     │ Modifier │ Type   │ Target                   │
├─────────────────┼──────────┼──────────┼────────┼──────────────────────────┤
│ example.com     │ /        │ None     │ Static │ /var/www/html            │
│ example.com     │ /api     │ None     │ Proxy  │ http://backend:3000      │
│ api.example.com │ /v1      │ None     │ Proxy  │ http://api-v1:8080       │
│ api.example.com │ /static  │ PrefixP… │ Static │ /var/www/static          │
╰─────────────────┴──────────┴──────────┴────────┴──────────────────────────╯

Total: 4 locations
```

**Show only proxy locations:**

```bash
sudo nginx-discover extract locations --proxy-only
```

**Show only static file locations:**

```bash
sudo nginx-discover extract locations --static-only
```

**Filter by server:**

```bash
sudo nginx-discover extract locations --server api.example.com
```

**Generate service map:**

```bash
# Export all proxy locations as JSON for service discovery
sudo nginx-discover extract locations --proxy-only --format json -o service-map.json
```

---

### `export` - Export Configuration

Export entire configuration to different formats.

#### Usage

```bash
nginx-discover export <FORMAT> [OPTIONS]
```

#### Formats

- `json` - Export as JSON
- `yaml` - Export as YAML

#### Options

```
-o, --output <FILE>    Output file (stdout if not specified)
    --pretty           Pretty-print output (for JSON/YAML)
```

#### Examples

**Export to JSON:**

```bash
sudo nginx-discover export json --pretty
```

**Export to file:**

```bash
sudo nginx-discover export json --pretty --output config.json
sudo nginx-discover export yaml --output config.yaml
```

**Compact JSON (no pretty-print):**

```bash
sudo nginx-discover export json > config.min.json
```

**Use exported config:**

```bash
# Export and process with jq
sudo nginx-discover export json | jq '.directives[] | select(.name == "server")'

# Export and convert YAML to JSON
sudo nginx-discover export yaml | yq eval -o=json > config.json
```

---

### `doctor` - Health Check

Run diagnostics and health checks on NGINX configuration.

#### Usage

```bash
nginx-discover doctor [OPTIONS]
```

#### Options

```
    --no-network    Skip network checks
    --fix           Attempt to fix issues automatically (not yet implemented)
```

#### Checks Performed

1. ✓ NGINX binary found
2. ✓ Configuration file readable
3. ✓ Configuration syntax valid (nginx -t)
4. ✓ Configuration parseable by nginx-discovery
5. ✓ Log files and directories accessible
6. ✓ SSL certificates present

#### Examples

**Run all checks:**

```bash
sudo nginx-discover doctor
```

Output:
```
Running diagnostics...

✓ NGINX binary found: /usr/sbin/nginx (nginx version: nginx/1.24.0)
✓ Configuration file: /etc/nginx/nginx.conf
✓ Configuration syntax: valid
✓ Configuration parsed successfully
⚠ Log files: 1 warnings (Log directory does not exist: /var/log/custom/)
✓ SSL servers: 2 configured

=== Summary ===

  5 checks passed
  1 warnings
  0 errors

Passed with warnings
```

**Skip network checks:**

```bash
sudo nginx-discover doctor --no-network
```

**Exit codes:**

- `0` - All checks passed
- `1` - One or more checks failed

Use in scripts:
```bash
if sudo nginx-discover doctor --quiet; then
    echo "Configuration is healthy"
    sudo nginx -s reload
else
    echo "Configuration has issues!"
    exit 1
fi
```

---

## Examples

### DevOps Workflows

#### 1. Pre-Deployment Validation

```bash
#!/bin/bash
# validate-nginx.sh

echo "Validating NGINX configuration..."

# Run health check
if ! sudo nginx-discover doctor --quiet; then
    echo "❌ Configuration validation failed"
    exit 1
fi

# Check for SSL certificates
SSL_COUNT=$(sudo nginx-discover extract servers --ssl-only --format json | jq '. | length')
if [ "$SSL_COUNT" -gt 0 ]; then
    echo "✓ Found $SSL_COUNT SSL-enabled servers"
fi

# Extract and validate log paths
sudo nginx-discover extract logs --format json -o /tmp/logs.json
echo "✓ Configuration validated successfully"
```

#### 2. Infrastructure Inventory

```bash
#!/bin/bash
# inventory.sh - Generate infrastructure inventory

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
OUTPUT_DIR="inventory_$TIMESTAMP"

mkdir -p "$OUTPUT_DIR"

# Extract all information
sudo nginx-discover extract servers --format json -o "$OUTPUT_DIR/servers.json"
sudo nginx-discover extract logs --with-formats --format json -o "$OUTPUT_DIR/logs.json"
sudo nginx-discover extract locations --format json -o "$OUTPUT_DIR/locations.json"

# Generate summary
sudo nginx-discover parse --summary > "$OUTPUT_DIR/summary.txt"

# Create service map
sudo nginx-discover extract locations --proxy-only --format json -o "$OUTPUT_DIR/service-map.json"

echo "Inventory generated in $OUTPUT_DIR/"
```

#### 3. Log Collector Configuration

```bash
#!/bin/bash
# generate-fluentd-config.sh

# Extract log configurations
sudo nginx-discover extract logs --with-formats --format json -o nginx-logs.json

# Generate Fluentd configuration
cat > fluentd.conf << 'EOF'
# Auto-generated Fluentd configuration for NGINX logs
EOF

# Parse JSON and generate Fluentd sources
jq -r '.logs[] | @json' nginx-logs.json | while read -r log; do
    PATH=$(echo "$log" | jq -r '.path')
    FORMAT=$(echo "$log" | jq -r '.format_name // "combined"')

    cat >> fluentd.conf << EOF

<source>
  @type tail
  path $PATH
  tag nginx.access
  <parse>
    @type nginx
  </parse>
</source>
EOF
done

echo "Fluentd configuration generated: fluentd.conf"
```

#### 4. Security Audit

```bash
#!/bin/bash
# security-audit.sh

echo "=== NGINX Security Audit ==="
echo

# Check SSL configuration
echo "SSL/TLS Servers:"
sudo nginx-discover extract servers --ssl-only --format table

# Check for default servers
echo -e "\nChecking for default servers..."
sudo nginx-discover extract servers --format json | \
    jq '.[] | select(.listen[].default_server == true) | .server_names[0]'

# List all listening ports
echo -e "\nOpen Ports:"
sudo nginx-discover parse --summary | grep "Listening ports"

# Check for proxy locations
echo -e "\nProxy Endpoints:"
sudo nginx-discover extract locations --proxy-only --format table
```

#### 5. Configuration Monitoring

```bash
#!/bin/bash
# monitor-config-changes.sh

BASELINE="config-baseline.json"
CURRENT="config-current.json"

# Create baseline on first run
if [ ! -f "$BASELINE" ]; then
    sudo nginx-discover export json --pretty -o "$BASELINE"
    echo "Baseline created: $BASELINE"
    exit 0
fi

# Export current config
sudo nginx-discover export json --pretty -o "$CURRENT"

# Compare
if diff -q "$BASELINE" "$CURRENT" > /dev/null; then
    echo "✓ No configuration changes detected"
else
    echo "⚠ Configuration has changed!"
    echo
    echo "Changes:"
    diff -u "$BASELINE" "$CURRENT"

    # Update baseline
    cp "$CURRENT" "$BASELINE"
fi
```

### CI/CD Integration

#### GitHub Actions

```yaml
name: Validate NGINX Config

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3

    - name: Install NGINX
      run: sudo apt-get update && sudo apt-get install -y nginx

    - name: Install nginx-discover
      run: |
        cargo install --git https://github.com/urwithajit9/nginx-discovery --features cli

    - name: Run health check
      run: sudo nginx-discover doctor

    - name: Generate report
      run: |
        sudo nginx-discover extract servers --format json > servers.json
        sudo nginx-discover extract logs --format json > logs.json

    - name: Upload artifacts
      uses: actions/upload-artifact@v3
      with:
        name: nginx-config-report
        path: |
          servers.json
          logs.json
```

---

## Output Formats

### Table Format (Default)

Beautiful, human-readable tables with borders:

```bash
sudo nginx-discover extract servers
```

Best for: Terminal viewing, quick inspection

### JSON Format

Structured data for programmatic processing:

```bash
sudo nginx-discover extract servers --format json
```

Best for: APIs, automation, jq processing

Example:
```json
[
  {
    "server_names": ["example.com"],
    "listen": [
      {
        "address": "*",
        "port": 80,
        "ssl": false,
        "http2": false,
        "default_server": false
      }
    ],
    "locations": [...]
  }
]
```

### YAML Format

Human-readable structured data:

```bash
sudo nginx-discover extract servers --format yaml
```

Best for: Configuration files, documentation

Example:
```yaml
- server_names:
  - example.com
  listen:
  - address: "*"
    port: 80
    ssl: false
  locations: []
```

### CSV Format

Spreadsheet-compatible format:

```bash
sudo nginx-discover extract servers --format csv > servers.csv
```

Best for: Excel, data analysis, reporting

Example:
```csv
Server Name,Port,SSL,Locations,Default
example.com,80,No,3,No
api.example.com,443,Yes,5,No
```

---

## Tips & Tricks

### 1. Combine with Standard Tools

**Use with jq:**

```bash
# Count servers per port
sudo nginx-discover extract servers --format json | \
    jq 'group_by(.listen[0].port) | map({port: .[0].listen[0].port, count: length})'

# Find servers without SSL
sudo nginx-discover extract servers --format json | \
    jq '.[] | select(.listen[].ssl == false) | .server_names[0]'
```

**Use with grep:**

```bash
# Find all proxy locations
sudo nginx-discover extract locations | grep Proxy

# Count SSL servers
sudo nginx-discover extract servers --ssl-only | grep -c "│"
```

**Use with awk:**

```bash
# Extract just server names and ports
sudo nginx-discover extract servers --format csv | awk -F',' '{print $1, $2}'
```

### 2. Create Aliases

Add to your `.bashrc` or `.zshrc`:

```bash
alias ngx-check='sudo nginx-discover doctor'
alias ngx-servers='sudo nginx-discover extract servers'
alias ngx-logs='sudo nginx-discover extract logs --with-formats'
alias ngx-export='sudo nginx-discover export json --pretty'
```

### 3. Use in Scripts

```bash
#!/bin/bash

# Get server count
SERVER_COUNT=$(sudo nginx-discover extract servers --format json | jq '. | length')

# Get SSL server count
SSL_COUNT=$(sudo nginx-discover extract servers --ssl-only --format json | jq '. | length')

echo "Total servers: $SERVER_COUNT"
echo "SSL servers: $SSL_COUNT"
echo "HTTP-only: $((SERVER_COUNT - SSL_COUNT))"
```

### 4. Configuration Diffing

```bash
# Before changes
sudo nginx-discover export json --pretty > before.json

# Make changes to NGINX config

# After changes
sudo nginx-discover export json --pretty > after.json

# View differences
diff -u before.json after.json
```

### 5. Automated Documentation

```bash
#!/bin/bash
# generate-docs.sh

cat > NGINX_INFRASTRUCTURE.md << 'EOF'
# NGINX Infrastructure Documentation

Auto-generated on $(date)

## Server Summary

$(sudo nginx-discover parse --summary)

## All Servers

$(sudo nginx-discover extract servers --format table)

## Proxy Endpoints

$(sudo nginx-discover extract locations --proxy-only --format table)

## Log Configuration

$(sudo nginx-discover extract logs --with-formats --format table)
EOF

echo "Documentation generated: NGINX_INFRASTRUCTURE.md"
```

---

## Troubleshooting

### Permission Denied

**Problem:**
```
Error: Permission denied reading /etc/nginx/nginx.conf
```

**Solution:**
Run with `sudo`:
```bash
sudo nginx-discover parse
```

### Configuration File Not Found

**Problem:**
```
Error: Could not find NGINX configuration file
```

**Solution:**
Specify the config file path:
```bash
nginx-discover --config /path/to/nginx.conf parse
```

### nginx -t Fails in Doctor

**Problem:**
```
✗ Configuration syntax error: open() "/run/nginx.pid" failed (13: Permission denied)
```

**Explanation:**
This is expected if you're not running as root. The syntax check still validates the configuration structure.

**Solution:**
Run with `sudo`:
```bash
sudo nginx-discover doctor
```

### No Servers Found

**Problem:**
Parsing succeeds but no servers are extracted.

**Possible causes:**
1. Servers are defined in included files
2. Configuration uses unconventional structure

**Solution:**
```bash
# Check if includes are present
sudo nginx-discover parse --tree | grep include

# Manually check included files
sudo nginx-discover --config /etc/nginx/sites-enabled/default parse
```

### Color Output Issues

**Problem:**
Colors don't work or escape codes appear in output.

**Solution:**
```bash
# Force colors
nginx-discover --color always parse

# Disable colors
nginx-discover --color never parse

# Auto-detect (default)
nginx-discover --color auto parse
```

### Large Configuration Files

**Problem:**
Parsing is slow or output is too large.

**Solution:**
Use filters and output to files:
```bash
# Filter by server name
sudo nginx-discover extract servers --name "api.*" --format json -o api-servers.json

# Extract only what you need
sudo nginx-discover extract locations --proxy-only -o proxies.txt
```

---

## Getting Help

### Command Help

```bash
# General help
nginx-discover --help

# Command-specific help
nginx-discover parse --help
nginx-discover extract --help
nginx-discover extract servers --help
```

### Verbose Output

```bash
# See what the tool is doing
nginx-discover --verbose parse
```

### Report Issues

If you encounter a bug:

1. Run with `--verbose`
2. Include the output
3. Report at: https://github.com/urwithajit9/nginx-discovery/issues

### Community

- **GitHub**: https://github.com/urwithajit9/nginx-discovery
- **Documentation**: https://docs.rs/nginx-discovery
- **Crate**: https://crates.io/crates/nginx-discovery

---

## Advanced Usage

### Custom Config Paths

```bash
# Development config
nginx-discover --config ./nginx-dev.conf parse

# Production config
nginx-discover --config /etc/nginx/nginx.conf parse

# Test config
nginx-discover --config /tmp/test.conf doctor
```

### Quiet Mode for Scripts

```bash
#!/bin/bash
set -e

# Only output on error
if ! sudo nginx-discover --quiet doctor; then
    echo "Configuration validation failed!" >&2
    exit 1
fi

# Continue with deployment
echo "Deploying..."
```

### Chaining Commands

```bash
# Export, filter, and count
sudo nginx-discover extract servers --format json | \
    jq '.[] | select(.listen[].ssl == true)' | \
    jq -s '. | length'

# Generate report
{
    echo "# NGINX Configuration Report"
    echo
    sudo nginx-discover parse --summary
    echo
    echo "## Servers"
    sudo nginx-discover extract servers --format table
} > report.txt
```

---

## Appendix

### Feature Matrix

| Feature | Supported | Notes |
|---------|-----------|-------|
| Parse configs | ✅ | Full support |
| Extract servers | ✅ | With filtering |
| Extract logs | ✅ | With formats |
| Extract locations | ✅ | With filtering |
| SSL analysis | ✅ | Basic support |
| Health checks | ✅ | Multiple checks |
| JSON export | ✅ | Full support |
| YAML export | ✅ | Full support |
| CSV export | ✅ | For tables |
| Include resolution | ⏳ | Planned v0.3.0 |
| Upstream extraction | ⏳ | Planned v0.3.0 |
| Security audit | ⏳ | Planned v0.4.0 |

### Performance Tips

- Use filters to reduce output size
- Redirect large outputs to files
- Use `--quiet` in automated scripts
- Consider caching exported configs

### Version History

- **v0.2.1** - CLI implementation, Phase 1 complete
- **v0.2.0** - Server extraction support
- **v0.1.0** - Initial release, log extraction

---

**Need more help?** Check the [main documentation](https://docs.rs/nginx-discovery) or [open an issue](https://github.com/urwithajit9/nginx-discovery/issues).