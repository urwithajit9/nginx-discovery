# nginx-discover Quick Reference

## Installation

```bash
cargo install --path . --features cli
# or
sudo cp ./target/release/nginx-discover /usr/local/bin/
```

## Quick Commands

```bash
# Health check
sudo nginx-discover doctor

# Parse config
sudo nginx-discover parse --tree

# Show servers
sudo nginx-discover extract servers

# Show logs
sudo nginx-discover extract logs --with-formats

# Export to JSON
sudo nginx-discover export json --pretty
```

## All Commands

### parse
```bash
nginx-discover parse [--tree|--summary|--json]
```

### extract servers
```bash
nginx-discover extract servers [--ssl-only] [--port PORT] [--name NAME]
                                [-f FORMAT] [-o FILE]
```

### extract logs
```bash
nginx-discover extract logs [--with-formats] [--context CTX]
                            [-f FORMAT] [-o FILE]
```

### extract locations
```bash
nginx-discover extract locations [--proxy-only|--static-only]
                                  [--server NAME] [-f FORMAT] [-o FILE]
```

### export
```bash
nginx-discover export {json|yaml} [--pretty] [-o FILE]
```

### doctor
```bash
nginx-discover doctor [--no-network] [--fix]
```

## Global Options

```bash
-c, --config PATH    # Specify config file
-v, --verbose        # Verbose output
-q, --quiet          # Quiet mode
--color WHEN         # Color: auto|always|never
-h, --help           # Show help
-V, --version        # Show version
```

## Output Formats

```bash
-f, --format FORMAT  # table (default), json, yaml, csv
-o, --output FILE    # Write to file
```

## Common Workflows

### Pre-deployment Check
```bash
sudo nginx-discover doctor && echo "Ready to deploy"
```

### Generate Inventory
```bash
sudo nginx-discover extract servers -f json -o servers.json
sudo nginx-discover extract logs -f json -o logs.json
```

### Security Audit
```bash
sudo nginx-discover extract servers --ssl-only
sudo nginx-discover extract locations --proxy-only
```

### Service Discovery
```bash
sudo nginx-discover extract locations --proxy-only -f json | \
    jq '.[] | {server, path, upstream: .proxy_pass}'
```

## Useful Aliases

```bash
alias ngx-check='sudo nginx-discover doctor'
alias ngx-servers='sudo nginx-discover extract servers'
alias ngx-logs='sudo nginx-discover extract logs --with-formats'
alias ngx-ssl='sudo nginx-discover extract servers --ssl-only'
alias ngx-proxies='sudo nginx-discover extract locations --proxy-only'
```

## Exit Codes

- `0` - Success
- `1` - Error

## Tips

- Most commands need `sudo` for file access
- Use `-q` for scripts: `nginx-discover -q doctor`
- Pipe to jq for JSON processing
- Use `--color never` when piping output
- Combine with grep, awk, jq for powerful queries

## Examples with jq

```bash
# Count servers per port
sudo nginx-discover extract servers -f json | \
    jq 'group_by(.listen[0].port) | map({port: .[0].listen[0].port, count: length})'

# List all proxy upstreams
sudo nginx-discover extract locations --proxy-only -f json | \
    jq -r '.[].proxy_pass' | sort -u

# Find servers without SSL
sudo nginx-discover extract servers -f json | \
    jq '.[] | select(.listen[].ssl == false) | .server_names[0]'
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Permission denied | Use `sudo` |
| Config not found | Use `--config PATH` |
| No color | Use `--color always` |
| Too much output | Use `-f json -o file.json` |

## More Help

```bash
nginx-discover --help
nginx-discover COMMAND --help
```

Full documentation: [CLI_GUIDE.md](CLI_GUIDE.md)