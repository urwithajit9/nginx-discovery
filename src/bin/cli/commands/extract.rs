//! Extract command implementation

use crate::cli::args::{ExtractArgs, ExtractTarget, GlobalOpts, OutputFormat};
use crate::cli::output::table;
use crate::cli::utils;
use anyhow::{Context, Result};
use nginx_discovery::NginxDiscovery;
use std::fs;

pub fn run(args: ExtractArgs, global: &GlobalOpts) -> Result<()> {
    utils::setup_colors(global.color.clone());

    // Load configuration
    let config_path = utils::find_config(global)?;
    let discovery =
        NginxDiscovery::from_config_file(&config_path).context("Failed to parse configuration")?;

    // Extract based on target
    let (output, _format_arg, output_arg) = match args.target {
        ExtractTarget::Servers {
            ssl_only,
            port,
            name,
            format,
            output,
        } => {
            let fmt = format.unwrap_or(args.format);
            let out = output.or(args.output);
            (
                extract_servers(&discovery, &fmt, ssl_only, port, name.as_deref())?,
                fmt,
                out,
            )
        }
        ExtractTarget::Logs {
            with_formats,
            context,
            format,
            output,
        } => {
            let fmt = format.unwrap_or(args.format);
            let out = output.or(args.output);
            (
                extract_logs(&discovery, &fmt, with_formats, context.as_deref())?,
                fmt,
                out,
            )
        }
        ExtractTarget::Locations {
            proxy_only,
            static_only,
            server,
            format,
            output,
        } => {
            let fmt = format.unwrap_or(args.format);
            let out = output.or(args.output);
            (
                extract_locations(&discovery, &fmt, proxy_only, static_only, server.as_deref())?,
                fmt,
                out,
            )
        }
    };

    // Write output
    if let Some(output_path) = &output_arg {
        fs::write(output_path, &output)
            .with_context(|| format!("Failed to write to {}", output_path.display()))?;

        if !global.quiet {
            eprintln!("Output written to: {}", output_path.display());
        }
    } else {
        println!("{}", output);
    }

    Ok(())
}

fn extract_servers(
    discovery: &NginxDiscovery,
    format: &OutputFormat,
    ssl_only: bool,
    port_filter: Option<u16>,
    name_filter: Option<&str>,
) -> Result<String> {
    let mut servers = discovery.servers();

    // Apply filters
    if ssl_only {
        servers.retain(|s| s.has_ssl());
    }

    if let Some(port) = port_filter {
        servers.retain(|s| s.listen.iter().any(|l| l.port == port));
    }

    if let Some(name_pattern) = name_filter {
        servers.retain(|s| {
            s.server_names
                .iter()
                .any(|n| wildcard_match(name_pattern, n))
        });
    }

    match format {
        OutputFormat::Table => Ok(table::format_servers(&servers)),
        OutputFormat::Json => {
            serde_json::to_string_pretty(&servers).context("Failed to serialize to JSON")
        }
        OutputFormat::Yaml => {
            serde_yaml::to_string(&servers).context("Failed to serialize to YAML")
        }
        OutputFormat::Csv => Ok(table::format_servers_csv(&servers)),
    }
}

fn extract_logs(
    discovery: &NginxDiscovery,
    format: &OutputFormat,
    with_formats: bool,
    context_filter: Option<&str>,
) -> Result<String> {
    let mut logs = discovery.access_logs();

    // Apply context filter
    if let Some(context) = context_filter {
        logs.retain(|log| {
            format!("{:?}", log.context)
                .to_lowercase()
                .contains(&context.to_lowercase())
        });
    }

    let formats = if with_formats {
        Some(discovery.log_formats())
    } else {
        None
    };

    match format {
        OutputFormat::Table => Ok(table::format_logs(&logs, formats.as_deref())),
        OutputFormat::Json => {
            let data = if with_formats {
                serde_json::json!({
                    "logs": logs,
                    "formats": formats
                })
            } else {
                serde_json::json!({ "logs": logs })
            };
            serde_json::to_string_pretty(&data).context("Failed to serialize to JSON")
        }
        OutputFormat::Yaml => {
            let data = if with_formats {
                serde_yaml::to_string(&(logs, formats))
            } else {
                serde_yaml::to_string(&logs)
            };
            data.context("Failed to serialize to YAML")
        }
        OutputFormat::Csv => Ok(table::format_logs_csv(&logs)),
    }
}

fn extract_locations(
    discovery: &NginxDiscovery,
    format: &OutputFormat,
    proxy_only: bool,
    static_only: bool,
    server_filter: Option<&str>,
) -> Result<String> {
    let servers = discovery.servers();

    let mut locations = Vec::new();
    for server in servers {
        let server_name = server.primary_name().unwrap_or("_").to_string();

        // Apply server filter
        if let Some(filter) = server_filter {
            if !wildcard_match(filter, &server_name) {
                continue;
            }
        }

        for location in &server.locations {
            // Apply type filters
            if proxy_only && !location.is_proxy() {
                continue;
            }
            if static_only && !location.is_static() {
                continue;
            }

            locations.push((server_name.clone(), location.clone()));
        }
    }

    match format {
        OutputFormat::Table => Ok(table::format_locations(&locations)),
        OutputFormat::Json => {
            let data: Vec<_> = locations
                .iter()
                .map(|(server, loc)| {
                    serde_json::json!({
                        "server": server,
                        "path": loc.path,
                        "modifier": format!("{:?}", loc.modifier),
                        "proxy_pass": loc.proxy_pass,
                        "root": loc.root,
                    })
                })
                .collect();
            serde_json::to_string_pretty(&data).context("Failed to serialize to JSON")
        }
        OutputFormat::Yaml => {
            let data: Vec<_> = locations
                .iter()
                .map(|(server, loc)| (server, &loc.path, &loc.modifier, &loc.proxy_pass, &loc.root))
                .collect();
            serde_yaml::to_string(&data).context("Failed to serialize to YAML")
        }
        OutputFormat::Csv => Ok(table::format_locations_csv(&locations)),
    }
}

fn wildcard_match(pattern: &str, text: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    if pattern.starts_with('*') && pattern.ends_with('*') {
        let middle = &pattern[1..pattern.len() - 1];
        return text.contains(middle);
    }

    if let Some(suffix) = pattern.strip_prefix('*') {
        return text.ends_with(suffix);
    }

    if let Some(prefix) = pattern.strip_suffix('*') {
        return text.starts_with(prefix);
    }

    pattern == text
}
