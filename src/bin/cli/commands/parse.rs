//! Parse command implementation

use crate::cli::args::{GlobalOpts, ParseArgs};
use crate::cli::output::tree;
use crate::cli::utils;
use anyhow::{Context, Result};
use colored::Colorize;
use nginx_discovery::NginxDiscovery;

pub fn run(args: ParseArgs, global: &GlobalOpts) -> Result<()> {
    utils::setup_colors(global.color.clone());

    // Load configuration
    let config_path = utils::find_config(global)?;
    if global.verbose {
        eprintln!("{} {}", "Reading config:".dimmed(), config_path.display());
    }

    let discovery = if config_path.exists() {
        NginxDiscovery::from_config_file(&config_path).context("Failed to parse configuration")?
    } else {
        anyhow::bail!("Configuration file not found: {}", config_path.display());
    };

    // Output based on format
    if args.json {
        let json = discovery.to_json().context("Failed to serialize to JSON")?;
        println!("{}", json);
    } else if args.tree {
        tree::print_tree(discovery.config());
    } else if args.summary {
        print_summary(&discovery);
    } else {
        // Default: both tree and summary
        tree::print_tree(discovery.config());
        println!();
        print_summary(&discovery);
    }

    if !global.quiet {
        println!("\n{}", "✓ Configuration parsed successfully".green().bold());
    }

    Ok(())
}

fn print_summary(discovery: &NginxDiscovery) {
    println!("\n{}", "=== Configuration Summary ===".bold());
    println!();
    println!(
        "  Total directives: {}",
        discovery.config().count_directives()
    );

    let servers = discovery.servers();
    println!("  Server blocks: {}", servers.len());

    if !servers.is_empty() {
        let ssl_count = discovery.ssl_servers().len();
        println!("    - SSL enabled: {}", ssl_count);
        println!("    - HTTP only: {}", servers.len() - ssl_count);
    }

    let ports = discovery.listening_ports();
    if !ports.is_empty() {
        println!("  Listening ports: {:?}", ports);
    }

    let logs = discovery.access_logs();
    println!("  Access logs: {}", logs.len());

    let formats = discovery.log_formats();
    println!("  Log formats: {}", formats.len());

    let location_count = discovery.location_count();
    if location_count > 0 {
        println!("  Location blocks: {}", location_count);

        let proxies = discovery.proxy_locations();
        if !proxies.is_empty() {
            println!("    - Proxy locations: {}", proxies.len());
        }
    }
}
