//! Export command implementation

use crate::cli::args::{ExportArgs, ExportFormat, GlobalOpts};
use crate::cli::utils;
use anyhow::{Context, Result};
use nginx_discovery::NginxDiscovery;
use std::fs;

pub fn run(args: ExportArgs, global: &GlobalOpts) -> Result<()> {
    utils::setup_colors(global.color.clone());

    // Load configuration
    let config_path = utils::find_config(global)?;
    let discovery =
        NginxDiscovery::from_config_file(&config_path).context("Failed to parse configuration")?;

    // Export based on format
    let output = match args.format {
        ExportFormat::Json => {
            if args.pretty {
                discovery.to_json().context("Failed to export to JSON")?
            } else {
                serde_json::to_string(discovery.config()).context("Failed to export to JSON")?
            }
        }
        ExportFormat::Yaml => discovery.to_yaml().context("Failed to export to YAML")?,
    };

    // Write output
    if let Some(output_path) = &args.output {
        fs::write(output_path, &output)
            .with_context(|| format!("Failed to write to {}", output_path.display()))?;

        if !global.quiet {
            eprintln!("Configuration exported to: {}", output_path.display());
        }
    } else {
        println!("{}", output);
    }

    Ok(())
}
