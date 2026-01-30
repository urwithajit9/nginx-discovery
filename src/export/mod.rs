// src/export/mod.rs
//! Enhanced export functionality with multiple format support.
//!
//! This module provides comprehensive export capabilities for NGINX configurations,
//! supporting JSON, YAML, TOML, and Markdown formats with filtering and customization.
//!
//! # Features
//!
//! - Multiple export formats (JSON, YAML, TOML, Markdown)
//! - Pretty printing and compact modes
//! - Export filtering (by server name, port, SSL status, etc.)
//! - Metadata inclusion
//! - Builder pattern for flexible options
//!
//! # Examples
//!
//! Basic export to JSON:
//!
//! ```no_run
//! use nginx_discovery::{parse, export::{export, ExportOptions, ExportFormat}};
//! use std::io;
//!
//! let config = parse("server { listen 80; }")?;
//! let options = ExportOptions::builder()
//!     .format(ExportFormat::Json)
//!     .pretty(true)
//!     .build();
//!
//! export(&config, &mut io::stdout(), &options)?;
//! # Ok::<(), nginx_discovery::Error>(())
//! ```
//!
//! Export with filtering:
//!
//! ```no_run
//! use nginx_discovery::{parse, export::{export, ExportOptions, ExportFormat, Filter, FilterType}};
//! use std::io;
//!
//! let config = parse("server { listen 80; server_name example.com; }")?;
//! let filter = Filter::new(FilterType::ServerName, "example.com");
//! let options = ExportOptions::builder()
//!     .format(ExportFormat::Json)
//!     .filter(filter)
//!     .build();
//!
//! export(&config, &mut io::stdout(), &options)?;
//! # Ok::<(), nginx_discovery::Error>(())
//! ```

pub mod filter;
pub mod format;
pub mod options;

pub use filter::{Filter, FilterType};
pub use format::{ExportFormat, Exporter};
pub use options::{ExportOptions, ExportOptionsBuilder};

use crate::{ast::Config, Result};
use std::io::Write;

/// Exports an NGINX configuration with the specified options.
///
/// This is the main export function that handles all export formats and filtering.
/// It applies any specified filters and then exports the configuration in the
/// requested format.
///
/// # Arguments
///
/// * `config` - The NGINX configuration to export
/// * `writer` - The destination for the exported output
/// * `options` - Export options controlling format, filtering, and output style
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if:
/// - The filter application fails
/// - The serialization fails
/// - Writing to the output fails
/// - A required feature is not enabled
///
/// # Errors
///
/// Returns an error if:
/// - Filtering fails (invalid filter pattern, etc.)
/// - Serialization fails (configuration cannot be represented in the target format)
/// - I/O errors occur while writing
/// - Required feature is not enabled (e.g., `export-toml` for TOML format)
///
/// # Examples
///
/// ```no_run
/// use nginx_discovery::{parse, export::{export, ExportOptions, ExportFormat}};
/// use std::fs::File;
///
/// let config = parse("server { listen 80; }")?;
/// let mut file = File::create("output.json")?;
/// let options = ExportOptions::builder()
///     .format(ExportFormat::Json)
///     .pretty(true)
///     .build();
///
/// export(&config, &mut file, &options)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn export<W: Write>(config: &Config, writer: &mut W, options: &ExportOptions) -> Result<()> {
    // Apply filters if specified
    let filtered_config = if let Some(filter) = &options.filter {
        filter.apply(config)?
    } else {
        config.clone()
    };

    // Export using specified format
    match options.format {
        ExportFormat::Json => {
            export_json(&filtered_config, writer, options)?;
        }
        ExportFormat::Yaml => {
            export_yaml(&filtered_config, writer, options)?;
        }
        #[cfg(feature = "export-toml")]
        ExportFormat::Toml => {
            export_toml(&filtered_config, writer, options)?;
        }
        #[cfg(feature = "export-markdown")]
        ExportFormat::Markdown => {
            export_markdown(&filtered_config, writer, options)?;
        }
    }

    Ok(())
}

/// Exports configuration to JSON format.
///
/// Supports both pretty-printed and compact output based on options.
fn export_json<W: Write>(config: &Config, writer: &mut W, options: &ExportOptions) -> Result<()> {
    #[cfg(feature = "serde")]
    {
        let json = if options.pretty {
            serde_json::to_string_pretty(config)?
        } else {
            serde_json::to_string(config)?
        };
        writer.write_all(json.as_bytes())?;
        Ok(())
    }
    #[cfg(not(feature = "serde"))]
    {
        let _ = (config, options);
        Err(crate::Error::FeatureNotEnabled("serde".to_string()))
    }
}

/// Exports configuration to YAML format.
///
/// YAML is always pretty-printed by default.
fn export_yaml<W: Write>(config: &Config, writer: &mut W, _options: &ExportOptions) -> Result<()> {
    #[cfg(feature = "serde")]
    {
        let yaml = serde_yaml::to_string(config)?;
        writer.write_all(yaml.as_bytes())?;
        Ok(())
    }
    #[cfg(not(feature = "serde"))]
    {
        let _ = config;
        Err(crate::Error::FeatureNotEnabled("serde".to_string()))
    }
}

/// Exports configuration to TOML format.
///
/// Requires the `export-toml` feature.
/// Supports both pretty-printed and compact output.
#[cfg(feature = "export-toml")]
fn export_toml<W: Write>(config: &Config, writer: &mut W, options: &ExportOptions) -> Result<()> {
    let toml_str = if options.pretty {
        toml::to_string_pretty(config)?
    } else {
        toml::to_string(config)?
    };
    writer.write_all(toml_str.as_bytes())?;
    Ok(())
}

/// Exports configuration to Markdown format for documentation.
///
/// Requires the `export-markdown` feature.
/// Generates a human-readable report with sections for servers, upstreams, etc.
#[cfg(feature = "export-markdown")]
fn export_markdown<W: Write>(
    config: &Config,
    writer: &mut W,
    options: &ExportOptions,
) -> Result<()> {
    use std::fmt::Write as FmtWrite;

    let mut md = String::new();

    // Title
    writeln!(md, "# NGINX Configuration Report")?;
    writeln!(md)?;

    // Metadata
    if options.include_metadata {
        writeln!(md, "## Metadata")?;
        writeln!(md)?;
        writeln!(
            md,
            "- **Generated**: {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        )?;
        writeln!(md, "- **Directives**: {}", config.directives.len())?;
        writeln!(md)?;
    }

    // HTTP Configuration - extract servers directly
    writeln!(md, "## HTTP Configuration")?;
    writeln!(md)?;

    // Servers
    let servers = crate::extract::servers(config)?;
    if !servers.is_empty() {
        writeln!(md, "### Servers ({} total)", servers.len())?;
        writeln!(md)?;

        for (i, server) in servers.iter().enumerate() {
            writeln!(md, "#### Server {}", i + 1)?;
            writeln!(md)?;

            if !server.server_names.is_empty() {
                writeln!(md, "- **Server Names**: {}", server.server_names.join(", "))?;
            }

            if !server.listen.is_empty() {
                writeln!(
                    md,
                    "- **Listen**: {}",
                    server
                        .listen
                        .iter()
                        .map(|l| format!("{}:{}", l.address, l.port))
                        .collect::<Vec<_>>()
                        .join(", ")
                )?;
            }

            if let Some(root) = &server.root {
                writeln!(md, "- **Root**: {}", root.display())?;
            }

            writeln!(md)?;
        }
    }

    // Upstreams
    writeln!(md, "### Upstreams")?;
    writeln!(md)?;
    writeln!(md, "_Upstream extraction coming in next version_")?;
    writeln!(md)?;

    // Stream block
    writeln!(md, "## Stream Configuration")?;
    writeln!(md)?;
    writeln!(md, "_Stream configuration support coming in next version_")?;
    writeln!(md)?;

    // Write to output
    writer.write_all(md.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_json() {
        let config = Config::default();
        let options = ExportOptions::default();
        let mut output = Vec::new();

        export(&config, &mut output, &options).unwrap();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_export_with_pretty() {
        let config = Config::default();
        let options = ExportOptions::builder().pretty(true).build();
        let mut output = Vec::new();

        export(&config, &mut output, &options).unwrap();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_export_compact() {
        let config = Config::default();
        let options = ExportOptions::builder().pretty(false).build();
        let mut output = Vec::new();

        export(&config, &mut output, &options).unwrap();
        assert!(!output.is_empty());
    }
}
