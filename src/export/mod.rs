// src/export/mod.rs
//! Enhanced export functionality with multiple format support

pub mod format;
pub mod options;
pub mod filter;

pub use format::{ExportFormat, Exporter};
pub use options::{ExportOptions, ExportOptionsBuilder};
pub use filter::{Filter, FilterType};

use crate::{ast::Config, Result};
use std::io::Write;

/// Main export function with full options support
pub fn export<W: Write>(config: &Config, writer: &mut W, options: ExportOptions) -> Result<()> {
    // Apply filters if specified
    let filtered_config = if let Some(filter) = &options.filter {
        filter.apply(config)?
    } else {
        config.clone()
    };

    // Export using specified format
    match options.format {
        ExportFormat::Json => {
            export_json(&filtered_config, writer, &options)?;
        }
        ExportFormat::Yaml => {
            export_yaml(&filtered_config, writer, &options)?;
        }
        #[cfg(feature = "export-toml")]
        ExportFormat::Toml => {
            export_toml(&filtered_config, writer, &options)?;
        }
        #[cfg(feature = "export-markdown")]
        ExportFormat::Markdown => {
            export_markdown(&filtered_config, writer, &options)?;
        }
    }

    Ok(())
}

/// Export to JSON format
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
        Err(crate::Error::FeatureNotEnabled("serde".to_string()))
    }
}

/// Export to YAML format
fn export_yaml<W: Write>(config: &Config, writer: &mut W, _options: &ExportOptions) -> Result<()> {
    #[cfg(feature = "serde")]
    {
        let yaml = serde_yaml::to_string(config)?;
        writer.write_all(yaml.as_bytes())?;
        Ok(())
    }
    #[cfg(not(feature = "serde"))]
    {
        Err(crate::Error::FeatureNotEnabled("serde".to_string()))
    }
}

/// Export to TOML format
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

/// Export to Markdown format (documentation/report style)
#[cfg(feature = "export-markdown")]
fn export_markdown<W: Write>(config: &Config, writer: &mut W, options: &ExportOptions) -> Result<()> {
    use std::fmt::Write as FmtWrite;

    let mut md = String::new();

    // Title
    writeln!(md, "# NGINX Configuration Report")?;
    writeln!(md)?;

    // Metadata
    if options.include_metadata {
        writeln!(md, "## Metadata")?;
        writeln!(md)?;
        writeln!(md, "- **Generated**: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"))?;
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
                writeln!(md, "- **Listen**: {}",
                    server.listen.iter()
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

        export(&config, &mut output, options).unwrap();
        assert!(!output.is_empty());
    }
}