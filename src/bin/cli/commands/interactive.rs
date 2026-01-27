//! Interactive mode implementation

use crate::cli::args::GlobalOpts;
use crate::cli::utils;
use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use nginx_discovery::NginxDiscovery;
use std::path::PathBuf;

pub fn run(global: &GlobalOpts) -> Result<()> {
    utils::setup_colors(global.color.clone());

    println!(
        "{}",
        "=== nginx-discover Interactive Mode ===".bold().blue()
    );
    println!();

    // Step 1: Select or input config file
    let config_path = select_config_file(global)?;

    println!("\n{} {}", "Loading:".dimmed(), config_path.display());

    let discovery =
        NginxDiscovery::from_config_file(&config_path).context("Failed to parse configuration")?;

    println!("{}", "✓ Configuration loaded successfully".green());

    // Main loop
    loop {
        println!();
        let action = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("What would you like to do?")
            .items(&[
                "📋 View Summary",
                "🖥️  List Servers",
                "📝 List Logs",
                "📍 List Locations",
                "🔒 Analyze SSL",
                "🛡️  Analyze Security",
                "💾 Export Configuration",
                "🏥 Run Health Check",
                "🔄 Reload Configuration",
                "❌ Exit",
            ])
            .default(0)
            .interact()?;

        match action {
            0 => show_summary(&discovery)?,
            1 => list_servers(&discovery)?,
            2 => list_logs(&discovery)?,
            3 => list_locations(&discovery)?,
            4 => analyze_ssl_interactive(&discovery)?,
            5 => analyze_security_interactive(&discovery)?,
            6 => export_interactive(&discovery)?,
            7 => run_health_check(&config_path)?,
            8 => {
                // Reload
                let new_discovery = NginxDiscovery::from_config_file(&config_path)
                    .context("Failed to reload configuration")?;
                println!("{}", "✓ Configuration reloaded".green());
                return run_with_discovery(new_discovery, &config_path);
            }
            9 => {
                println!("\n{}", "Goodbye! 👋".bold());
                break;
            }
            _ => unreachable!(),
        }

        // Ask if user wants to continue
        println!();
        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Continue?")
            .default(true)
            .interact()?
        {
            println!("\n{}", "Goodbye! 👋".bold());
            break;
        }
    }

    Ok(())
}

fn run_with_discovery(discovery: NginxDiscovery, config_path: &PathBuf) -> Result<()> {
    loop {
        println!();
        let action = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("What would you like to do?")
            .items(&[
                "📋 View Summary",
                "🖥️  List Servers",
                "📝 List Logs",
                "📍 List Locations",
                "🔒 Analyze SSL",
                "🛡️  Analyze Security",
                "💾 Export Configuration",
                "🏥 Run Health Check",
                "🔄 Reload Configuration",
                "❌ Exit",
            ])
            .default(0)
            .interact()?;

        match action {
            0 => show_summary(&discovery)?,
            1 => list_servers(&discovery)?,
            2 => list_logs(&discovery)?,
            3 => list_locations(&discovery)?,
            4 => analyze_ssl_interactive(&discovery)?,
            5 => analyze_security_interactive(&discovery)?,
            6 => export_interactive(&discovery)?,
            7 => run_health_check(config_path)?,
            8 => {
                let new_discovery = NginxDiscovery::from_config_file(config_path)?;
                println!("{}", "✓ Configuration reloaded".green());
                return run_with_discovery(new_discovery, config_path);
            }
            9 => {
                println!("\n{}", "Goodbye! 👋".bold());
                break;
            }
            _ => unreachable!(),
        }

        println!();
        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Continue?")
            .default(true)
            .interact()?
        {
            println!("\n{}", "Goodbye! 👋".bold());
            break;
        }
    }

    Ok(())
}

fn select_config_file(global: &GlobalOpts) -> Result<PathBuf> {
    if let Some(ref path) = global.config {
        return Ok(path.clone());
    }

    println!("{}", "Select NGINX configuration file:".bold());
    println!();

    let options = vec![
        "/etc/nginx/nginx.conf",
        "/usr/local/nginx/conf/nginx.conf",
        "/usr/local/etc/nginx/nginx.conf",
        "Custom path...",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Config file")
        .items(&options)
        .default(0)
        .interact()?;

    if selection == options.len() - 1 {
        // Custom path
        let path: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter config file path")
            .interact_text()?;
        Ok(PathBuf::from(path))
    } else {
        Ok(PathBuf::from(options[selection]))
    }
}

fn show_summary(discovery: &NginxDiscovery) -> Result<()> {
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

    Ok(())
}

fn list_servers(discovery: &NginxDiscovery) -> Result<()> {
    let servers = discovery.servers();

    if servers.is_empty() {
        println!("\n{}", "No servers found.".yellow());
        return Ok(());
    }

    println!(
        "\n{}",
        format!("=== Servers ({}) ===", servers.len()).bold()
    );

    for (i, server) in servers.iter().enumerate() {
        println!(
            "\n{} {}",
            format!("Server {}:", i + 1).bold(),
            server.primary_name().unwrap_or("_").blue()
        );

        if !server.server_names.is_empty() {
            println!("  Names: {}", server.server_names.join(", "));
        }

        if !server.listen.is_empty() {
            println!("  Listening:");
            for listen in &server.listen {
                let mut flags = Vec::new();
                if listen.ssl {
                    flags.push("SSL");
                }
                if listen.http2 {
                    flags.push("HTTP/2");
                }
                if listen.default_server {
                    flags.push("default");
                }

                let flags_str = if flags.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", flags.join(", "))
                };

                println!("    - {}:{}{}", listen.address, listen.port, flags_str);
            }
        }

        if let Some(root) = &server.root {
            println!("  Root: {}", root.display());
        }

        if !server.locations.is_empty() {
            println!("  Locations: {}", server.locations.len());
        }
    }

    Ok(())
}

fn list_logs(discovery: &NginxDiscovery) -> Result<()> {
    let logs = discovery.access_logs();

    if logs.is_empty() {
        println!("\n{}", "No log files found.".yellow());
        return Ok(());
    }

    println!(
        "\n{}",
        format!("=== Access Logs ({}) ===", logs.len()).bold()
    );

    for log in logs {
        println!("\n  {}", log.path.display().to_string().blue());
        if let Some(format_name) = &log.format_name {
            println!("    Format: {}", format_name);
        }
        println!("    Context: {:?}", log.context);
    }

    Ok(())
}

fn list_locations(discovery: &NginxDiscovery) -> Result<()> {
    let servers = discovery.servers();
    let mut total = 0;

    println!("\n{}", "=== Locations ===".bold());

    for server in servers {
        if server.locations.is_empty() {
            continue;
        }

        println!("\n  {}", server.primary_name().unwrap_or("_").blue().bold());

        for location in &server.locations {
            total += 1;
            let type_str = if location.is_proxy() {
                format!("Proxy → {}", location.proxy_pass.as_ref().unwrap()).yellow()
            } else if location.is_static() {
                "Static".green()
            } else {
                "Other".dimmed()
            };

            println!("    {} {}", location.path, type_str);
        }
    }

    println!("\n  Total: {} locations", total);

    Ok(())
}

fn analyze_ssl_interactive(discovery: &NginxDiscovery) -> Result<()> {
    let ssl_servers = discovery.ssl_servers();

    if ssl_servers.is_empty() {
        println!("\n{}", "No SSL/TLS configuration found.".yellow());
        return Ok(());
    }

    println!("\n{}", "=== SSL/TLS Analysis ===".bold());
    println!("\n  SSL-enabled servers: {}", ssl_servers.len());

    for server in ssl_servers {
        println!(
            "\n  {} {}",
            "✓".green(),
            server.primary_name().unwrap_or("_").bold()
        );

        for listen in &server.listen {
            if listen.ssl {
                let mut features = vec![format!("Port {}", listen.port)];
                if listen.http2 {
                    features.push("HTTP/2".to_string());
                }
                if listen.http3 {
                    features.push("HTTP/3".to_string());
                }
                println!("    {}", features.join(", "));
            }
        }
    }

    Ok(())
}

fn analyze_security_interactive(discovery: &NginxDiscovery) -> Result<()> {
    println!("\n{}", "=== Security Analysis ===".bold());
    println!(
        "\n{}",
        "Checking configuration for security issues...".dimmed()
    );

    let servers = discovery.servers();
    let mut issues = 0;

    // Check for default servers
    for server in &servers {
        if server.server_names.is_empty() && server.listen.iter().any(|l| l.default_server) {
            println!("\n  {} Default server without server_name", "⚠".yellow());
            issues += 1;
        }
    }

    // Check for unencrypted sensitive paths
    for server in &servers {
        if !server.has_ssl() {
            for location in &server.locations {
                if location.path.contains("/admin") || location.path.contains("/login") {
                    println!(
                        "\n  {} Sensitive path '{}' on HTTP server: {}",
                        "⚠".yellow(),
                        location.path,
                        server.primary_name().unwrap_or("_")
                    );
                    issues += 1;
                }
            }
        }
    }

    if issues == 0 {
        println!("\n  {} No obvious security issues found", "✓".green());
    } else {
        println!("\n  Found {} potential issues", issues);
    }

    Ok(())
}

fn export_interactive(discovery: &NginxDiscovery) -> Result<()> {
    let format = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Export format")
        .items(&["JSON", "YAML"])
        .default(0)
        .interact()?;

    let save_to_file = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Save to file?")
        .default(true)
        .interact()?;

    let output = match format {
        0 => discovery.to_json()?,
        1 => discovery.to_yaml()?,
        _ => unreachable!(),
    };

    if save_to_file {
        let filename: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Filename")
            .default(if format == 0 {
                "config.json".to_string()
            } else {
                "config.yaml".to_string()
            })
            .interact_text()?;

        std::fs::write(&filename, output)?;
        println!("\n{} Saved to: {}", "✓".green(), filename);
    } else {
        println!("\n{}", output);
    }

    Ok(())
}

fn run_health_check(_config_path: &PathBuf) -> Result<()> {
    println!("\n{}", "=== Health Check ===".bold());
    println!("\n{}", "Running diagnostics...".dimmed());

    // Basic checks
    println!("\n  {} Configuration file exists", "✓".green());
    println!("  {} Configuration is parseable", "✓".green());

    // Try nginx -t if available
    match nginx_discovery::system::test_config() {
        Ok(_) => println!("  {} NGINX syntax check passed", "✓".green()),
        Err(e) => {
            if e.to_string().contains("Permission denied") {
                println!(
                    "  {} NGINX syntax check skipped (requires sudo)",
                    "ℹ".blue()
                );
            } else {
                println!("  {} NGINX syntax check failed: {}", "✗".red(), e);
            }
        }
    }

    println!("\n{}", "Health check complete.".bold());

    Ok(())
}
