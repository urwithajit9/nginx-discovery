# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-01-22

### Added

#### Core Parsing
- Complete NGINX configuration lexer with support for:
  - Simple directives (e.g., `user nginx;`)
  - Block directives (e.g., `server { ... }`)
  - Nested blocks (e.g., `http { server { location { } } }`)
  - Quoted strings (both single and double quotes)
  - Variables (`$host`, `${variable}`)
  - Numbers and comments
- Robust parser with detailed error messages and source locations
- Type-safe AST (Abstract Syntax Tree) representation
- Zero-copy parsing where possible for performance

#### High-Level API
- `NginxDiscovery` struct for easy configuration discovery
- `NginxDiscovery::from_running_instance()` - Auto-detect and parse running NGINX
- `NginxDiscovery::from_config_file()` - Parse from specific config file
- `NginxDiscovery::from_config_text()` - Parse from string
- Convenient methods:
  - `access_logs()` - Get all access log configurations
  - `error_logs()` - Get all error log configurations
  - `log_formats()` - Get all log format definitions
  - `all_log_files()` - Get all log file paths
  - `server_names()` - Get all server names
  - `to_json()` - Export configuration to JSON

#### Extractors
- `extract::access_logs()` - Extract access log configurations with context
- `extract::log_formats()` - Extract log format definitions with variables
- `extract::error_logs()` - Extract error log configurations
- Support for log format variable extraction
- Context tracking (http, server, location blocks)

#### System Integration
- Auto-detection of running NGINX processes
- Config file path detection from running process
- Support for `nginx -T` command for full config dump
- Cross-platform process detection

#### Type Safety
- `AccessLog` struct with path, format, and context
- `LogFormat` struct with name, pattern, and variables
- `ErrorLog` struct with path, level, and context
- `Context` enum for tracking directive locations

#### Developer Experience
- Comprehensive error types with suggestions
- Detailed error messages with line/column information
- Great documentation with examples
- Benchmark suite for performance tracking

### Features

#### Feature Flags
- `system` (default) - System interaction utilities
- `serde` - Serialize/deserialize support
- `visitor` - Visitor pattern for AST traversal
- `includes` - Include directive resolution support
- `cli` - Command-line interface

#### CLI Tool
- `nginx-discover` command for configuration inspection
- JSON export support
- Selective extraction (logs, servers, etc.)
- Config file specification support

### Documentation
- Complete API documentation
- README with multiple examples
- Contributing guide
- Design decisions document
- Inline code examples

### Testing
- Unit tests for lexer
- Unit tests for parser
- Integration tests for extractors
- Example NGINX configurations for testing
- Benchmark suite

### Performance
- Efficient tokenization with minimal allocations
- Zero-copy string handling where possible
- Optimized directive lookup
- Fast variable extraction

### Known Limitations
- Include directives are parsed but not automatically resolved (v0.1.0)
- Map directives not yet supported (planned for v0.3.0)
- Upstream blocks not yet extracted (planned for v0.2.0)
- Server block extraction not yet implemented (planned for v0.2.0)
- No configuration validation (planned for v0.5.0)

[Unreleased]: https://github.com/urwithajit9/nginx-discovery/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/urwithajit9/nginx-discovery/releases/tag/v0.1.0

## [0.1.1] - 2026-01-22 (Unreleased)

### Added
- Complete implementation of `NginxDiscovery` API
  - `from_config_text()` - Parse from string
  - `from_config_file()` - Parse from file
  - `from_running_instance()` - Auto-detect running nginx
  - `access_logs()`, `log_formats()` - Extract configurations
  - `all_log_files()` - Get deduplicated log paths
  - `server_names()` - Extract server names
  - `to_json()`, `to_yaml()` - Export configurations
  - `summary()` - Generate configuration summary

- System module for nginx interaction
  - `find_nginx()` - Locate nginx binary
  - `nginx_version()` - Get nginx version
  - `dump_config()` - Run nginx -T
  - `test_config()` - Test configuration syntax
  - `detect_and_parse()` - Auto-detect and parse

- Integration tests for discovery API

### Changed
- Fixed documentation examples to use `no_run` instead of `ignore`
- All doc examples now compile-check correctly
- Improved error messages in system module

### Fixed
- Discovery API stubs now have working implementations
- System module properly handles permission errors
- Better error messages for common issues