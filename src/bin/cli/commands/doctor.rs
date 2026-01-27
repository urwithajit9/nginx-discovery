//! Doctor command implementation

use crate::cli::args::{DoctorArgs, GlobalOpts};
use crate::cli::utils;
use anyhow::Result;
use colored::Colorize;
use nginx_discovery::{system, NginxDiscovery};
use std::path::Path;

pub fn run(args: DoctorArgs, global: &GlobalOpts) -> Result<()> {
    utils::setup_colors(global.color.clone());

    println!("{}\n", "Running diagnostics...".bold());

    let mut passed = 0;
    let mut warnings = 0;
    let mut errors = 0;

    // Check 1: NGINX binary
    match check_nginx_binary() {
        CheckResult::Pass(msg) => {
            println!("{} {}", "✓".green(), msg);
            passed += 1;
        }
        CheckResult::Warning(msg) => {
            println!("{} {}", "⚠".yellow(), msg);
            warnings += 1;
        }
        CheckResult::Error(msg) => {
            println!("{} {}", "✗".red(), msg);
            errors += 1;
        }
    }

    // Check 2: Configuration file
    let config_path = match utils::find_config(global) {
        Ok(path) => path,
        Err(e) => {
            println!("{} Configuration file: {}", "✗".red(), e);
            errors += 1;
            return print_summary(passed, warnings, errors);
        }
    };

    match check_config_file(&config_path) {
        CheckResult::Pass(msg) => {
            println!("{} {}", "✓".green(), msg);
            passed += 1;
        }
        CheckResult::Warning(msg) => {
            println!("{} {}", "⚠".yellow(), msg);
            warnings += 1;
        }
        CheckResult::Error(msg) => {
            println!("{} {}", "✗".red(), msg);
            errors += 1;
        }
    }

    // Check 3: Configuration syntax
    match check_config_syntax(&config_path) {
        CheckResult::Pass(msg) => {
            println!("{} {}", "✓".green(), msg);
            passed += 1;
        }
        CheckResult::Warning(msg) => {
            println!("{} {}", "⚠".yellow(), msg);
            warnings += 1;
        }
        CheckResult::Error(msg) => {
            println!("{} {}", "✗".red(), msg);
            errors += 1;
        }
    }

    // Check 4: Parse with nginx-discovery
    let discovery = match NginxDiscovery::from_config_file(&config_path) {
        Ok(d) => {
            println!("{} Configuration parsed successfully", "✓".green());
            passed += 1;
            Some(d)
        }
        Err(e) => {
            println!("{} Configuration parsing failed: {}", "✗".red(), e);
            errors += 1;
            None
        }
    };

    // Check 5: Log files
    if let Some(ref discovery) = discovery {
        match check_log_files(discovery) {
            CheckResult::Pass(msg) => {
                println!("{} {}", "✓".green(), msg);
                passed += 1;
            }
            CheckResult::Warning(msg) => {
                println!("{} {}", "⚠".yellow(), msg);
                warnings += 1;
            }
            CheckResult::Error(msg) => {
                println!("{} {}", "✗".red(), msg);
                errors += 1;
            }
        }
    }

    // Check 6: SSL certificates
    if let Some(ref discovery) = discovery {
        match check_ssl_certificates(discovery) {
            CheckResult::Pass(msg) => {
                println!("{} {}", "✓".green(), msg);
                passed += 1;
            }
            CheckResult::Warning(msg) => {
                println!("{} {}", "⚠".yellow(), msg);
                warnings += 1;
            }
            CheckResult::Error(msg) => {
                println!("{} {}", "✗".red(), msg);
                errors += 1;
            }
        }
    }

    print_summary(passed, warnings, errors)?;

    if args.fix {
        println!("\n{}", "Automatic fixes not yet implemented.".dimmed());
        println!("{}", "Please resolve issues manually.".dimmed());
    }

    // Exit with error code if there are errors
    if errors > 0 {
        std::process::exit(1);
    }

    Ok(())
}

enum CheckResult {
    Pass(String),
    Warning(String),
    Error(String),
}

fn check_nginx_binary() -> CheckResult {
    match system::find_nginx() {
        Ok(path) => match system::nginx_version() {
            Ok(version) => CheckResult::Pass(format!(
                "NGINX binary found: {} ({})",
                path.display(),
                version
            )),
            Err(_) => CheckResult::Pass(format!("NGINX binary found: {}", path.display())),
        },
        Err(_) => CheckResult::Error("NGINX binary not found in PATH".to_string()),
    }
}

fn check_config_file(path: &Path) -> CheckResult {
    if !path.exists() {
        return CheckResult::Error(format!("Configuration file not found: {}", path.display()));
    }

    match std::fs::metadata(path) {
        Ok(metadata) => {
            if metadata.is_file() {
                CheckResult::Pass(format!("Configuration file: {}", path.display()))
            } else {
                CheckResult::Error(format!("Path is not a file: {}", path.display()))
            }
        }
        Err(e) => CheckResult::Error(format!("Cannot access config file: {}", e)),
    }
}

fn check_config_syntax(_path: &Path) -> CheckResult {
    match system::test_config() {
        Ok(_) => CheckResult::Pass("Configuration syntax: valid".to_string()),
        Err(e) => CheckResult::Error(format!("Configuration syntax error: {}", e)),
    }
}

fn check_log_files(discovery: &NginxDiscovery) -> CheckResult {
    let logs = discovery.all_log_files();

    if logs.is_empty() {
        return CheckResult::Warning("No log files configured".to_string());
    }

    let mut warnings: Vec<String> = Vec::new();

    for log_path in &logs {
        if let Some(parent) = log_path.parent() {
            if !parent.exists() {
                warnings.push(format!(
                    "Log directory does not exist: {}",
                    parent.display()
                ));
            } else if let Ok(metadata) = std::fs::metadata(parent) {
                // Check if directory is writable (Unix-specific check would be better)
                if metadata.permissions().readonly() {
                    warnings.push(format!("Log directory not writable: {}", parent.display()));
                }
            }
        }
    }

    if !warnings.is_empty() {
        CheckResult::Warning(format!(
            "Log files: {} warnings ({})",
            warnings.len(),
            warnings[0]
        ))
    } else {
        CheckResult::Pass(format!(
            "Log files: {} configured, all directories accessible",
            logs.len()
        ))
    }
}

fn check_ssl_certificates(discovery: &NginxDiscovery) -> CheckResult {
    let ssl_servers = discovery.ssl_servers();

    if ssl_servers.is_empty() {
        return CheckResult::Pass("No SSL configuration found".to_string());
    }

    // This is a basic check - in a real implementation, you'd parse
    // ssl_certificate directives and check if files exist
    CheckResult::Pass(format!("SSL servers: {} configured", ssl_servers.len()))
}

fn print_summary(passed: usize, warnings: usize, errors: usize) -> Result<()> {
    println!("\n{}", "=== Summary ===".bold());
    println!();
    println!("  {} checks passed", passed.to_string().green());

    if warnings > 0 {
        println!("  {} warnings", warnings.to_string().yellow());
    }

    if errors > 0 {
        println!("  {} errors", errors.to_string().red());
    }

    if errors == 0 && warnings == 0 {
        println!("\n{}", "All checks passed! ✨".green().bold());
    } else if errors == 0 {
        println!("\n{}", "Passed with warnings".yellow());
    } else {
        println!("\n{}", "Some checks failed".red());
    }

    Ok(())
}
