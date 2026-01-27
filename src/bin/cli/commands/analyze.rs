//! Analyze command implementation

use crate::cli::args::{AnalyzeArgs, AnalyzeTarget, GlobalOpts, OutputFormat};
use crate::cli::utils;
use anyhow::{Context, Result};
use colored::Colorize;
use nginx_discovery::NginxDiscovery;
use std::fs;

pub fn run(args: AnalyzeArgs, global: &GlobalOpts) -> Result<()> {
    utils::setup_colors(global.color.clone());

    // Load configuration
    let config_path = utils::find_config(global)?;
    let discovery =
        NginxDiscovery::from_config_file(&config_path).context("Failed to parse configuration")?;

    // Analyze based on target
    let (output, output_path) = match args.target {
        AnalyzeTarget::Ssl {
            warnings_only,
            check_certs,
            format,
            output,
        } => {
            let result = analyze_ssl(&discovery, &format, warnings_only, check_certs)?;
            (result, output)
        }
        AnalyzeTarget::Security {
            level,
            fix,
            format,
            output,
        } => {
            let result = analyze_security(&discovery, &format, &level, fix)?;
            (result, output)
        }
    };

    // Write output
    if let Some(path) = &output_path {
        fs::write(path, &output)
            .with_context(|| format!("Failed to write to {}", path.display()))?;

        if !global.quiet {
            eprintln!("Analysis written to: {}", path.display());
        }
    } else {
        println!("{}", output);
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct SslIssue {
    severity: Severity,
    server: String,
    issue: String,
    recommendation: String,
}

#[derive(Debug, Clone, PartialEq)]
enum Severity {
    Info,
    Warning,
    Critical,
}

fn analyze_ssl(
    discovery: &NginxDiscovery,
    format: &OutputFormat,
    warnings_only: bool,
    check_certs: bool,
) -> Result<String> {
    let ssl_servers = discovery.ssl_servers();

    if ssl_servers.is_empty() {
        return Ok("No SSL/TLS configuration found.".to_string());
    }

    let mut issues = Vec::new();

    for server in &ssl_servers {
        let server_name = server.primary_name().unwrap_or("_").to_string();

        // Check 1: SSL protocols
        check_ssl_protocols(&server_name, &mut issues);

        // Check 2: HTTP/2 support
        check_http2_support(server, &server_name, &mut issues);

        // Check 3: HSTS headers
        check_hsts(&server_name, &mut issues);

        // Check 4: Certificate files (if requested)
        if check_certs {
            check_certificate_files(&server_name, &mut issues);
        }

        // Check 5: Mixed content
        check_mixed_content(server, &server_name, &mut issues);
    }

    // Filter by severity
    if warnings_only {
        issues.retain(|i| i.severity != Severity::Info);
    }

    // Format output
    format_ssl_analysis(&ssl_servers, &issues, format)
}

fn check_ssl_protocols(server_name: &str, issues: &mut Vec<SslIssue>) {
    // This is a placeholder - in real implementation, we'd parse ssl_protocols directive
    // For now, we provide general guidance
    issues.push(SslIssue {
        severity: Severity::Info,
        server: server_name.to_string(),
        issue: "SSL/TLS protocol configuration not explicitly checked".to_string(),
        recommendation: "Ensure ssl_protocols directive uses TLSv1.2 and TLSv1.3 only".to_string(),
    });
}

fn check_http2_support(
    server: &nginx_discovery::types::Server,
    server_name: &str,
    issues: &mut Vec<SslIssue>,
) {
    let has_http2 = server.listen.iter().any(|l| l.ssl && l.http2);

    if !has_http2 {
        issues.push(SslIssue {
            severity: Severity::Warning,
            server: server_name.to_string(),
            issue: "HTTP/2 not enabled on SSL listener".to_string(),
            recommendation: "Add 'http2' parameter to listen directive: listen 443 ssl http2;"
                .to_string(),
        });
    }
}

fn check_hsts(_server_name: &str, _issues: &mut Vec<SslIssue>) {
    // Placeholder for HSTS header check
    // Would need to parse add_header directives
}

fn check_certificate_files(server_name: &str, issues: &mut Vec<SslIssue>) {
    // Placeholder - would need to parse ssl_certificate directives and check files
    issues.push(SslIssue {
        severity: Severity::Info,
        server: server_name.to_string(),
        issue: "Certificate file validation not implemented".to_string(),
        recommendation: "Manually verify ssl_certificate and ssl_certificate_key files exist"
            .to_string(),
    });
}

fn check_mixed_content(
    server: &nginx_discovery::types::Server,
    server_name: &str,
    issues: &mut Vec<SslIssue>,
) {
    // Check if any locations proxy to http:// (potential mixed content)
    for location in &server.locations {
        if let Some(proxy_pass) = &location.proxy_pass {
            if proxy_pass.starts_with("http://") {
                issues.push(SslIssue {
                    severity: Severity::Warning,
                    server: server_name.to_string(),
                    issue: format!(
                        "Location {} proxies to HTTP upstream (mixed content risk)",
                        location.path
                    ),
                    recommendation: format!("Consider using HTTPS for upstream: {}", proxy_pass),
                });
            }
        }
    }
}

fn format_ssl_analysis(
    servers: &[nginx_discovery::types::Server],
    issues: &[SslIssue],
    format: &OutputFormat,
) -> Result<String> {
    match format {
        OutputFormat::Table => {
            let mut output = String::new();

            output.push_str(&format!("{}\n\n", "=== SSL/TLS Analysis ===".bold()));
            output.push_str(&format!("SSL-enabled servers: {}\n\n", servers.len()));

            if issues.is_empty() {
                output.push_str(&format!("{}\n", "✓ No issues found".green()));
            } else {
                // Group by severity
                let critical: Vec<_> = issues
                    .iter()
                    .filter(|i| i.severity == Severity::Critical)
                    .collect();
                let warnings: Vec<_> = issues
                    .iter()
                    .filter(|i| i.severity == Severity::Warning)
                    .collect();
                let info: Vec<_> = issues
                    .iter()
                    .filter(|i| i.severity == Severity::Info)
                    .collect();

                if !critical.is_empty() {
                    output.push_str(&format!("\n{}\n", "CRITICAL ISSUES:".red().bold()));
                    for issue in &critical {
                        output.push_str(&format!("\n{} {}\n", "✗".red(), issue.server.bold()));
                        output.push_str(&format!("  Issue: {}\n", issue.issue));
                        output.push_str(&format!("  Fix: {}\n", issue.recommendation.dimmed()));
                    }
                }

                if !warnings.is_empty() {
                    output.push_str(&format!("\n{}\n", "WARNINGS:".yellow().bold()));
                    for issue in &warnings {
                        output.push_str(&format!("\n{} {}\n", "⚠".yellow(), issue.server.bold()));
                        output.push_str(&format!("  Issue: {}\n", issue.issue));
                        output.push_str(&format!("  Fix: {}\n", issue.recommendation.dimmed()));
                    }
                }

                if !info.is_empty() {
                    output.push_str(&format!("\n{}\n", "INFORMATION:".blue().bold()));
                    for issue in &info {
                        output.push_str(&format!("\n{} {}\n", "ℹ".blue(), issue.server.bold()));
                        output.push_str(&format!("  {}\n", issue.issue));
                        output.push_str(&format!("  {}\n", issue.recommendation.dimmed()));
                    }
                }

                output.push_str(&format!(
                    "\n{}\n  {} critical, {} warnings, {} info\n",
                    "Summary:".bold(),
                    critical.len(),
                    warnings.len(),
                    info.len()
                ));
            }

            Ok(output)
        }
        OutputFormat::Json => {
            let data = serde_json::json!({
                "ssl_servers_count": servers.len(),
                "issues": issues.iter().map(|i| {
                    serde_json::json!({
                        "severity": format!("{:?}", i.severity),
                        "server": i.server,
                        "issue": i.issue,
                        "recommendation": i.recommendation,
                    })
                }).collect::<Vec<_>>()
            });
            serde_json::to_string_pretty(&data).context("Failed to serialize")
        }
        OutputFormat::Yaml => {
            let data = issues
                .iter()
                .map(|i| {
                    (
                        format!("{:?}", i.severity),
                        &i.server,
                        &i.issue,
                        &i.recommendation,
                    )
                })
                .collect::<Vec<_>>();
            serde_yaml::to_string(&data).context("Failed to serialize")
        }
        OutputFormat::Csv => {
            let mut output = String::from("Severity,Server,Issue,Recommendation\n");
            for issue in issues {
                output.push_str(&format!(
                    "{:?},{},{},{}\n",
                    issue.severity, issue.server, issue.issue, issue.recommendation
                ));
            }
            Ok(output)
        }
    }
}

fn analyze_security(
    discovery: &NginxDiscovery,
    format: &OutputFormat,
    level: &str,
    show_fix: bool,
) -> Result<String> {
    let servers = discovery.servers();
    let mut issues = Vec::new();

    for server in &servers {
        let server_name = server.primary_name().unwrap_or("_").to_string();

        // Check 1: Default server without server_name
        if (server.server_names.is_empty() || server.server_names.contains(&"_".to_string()))
            && server.listen.iter().any(|l| l.default_server)
        {
            issues.push(SecurityIssue {
                severity: Severity::Warning,
                server: server_name.clone(),
                category: "Configuration".to_string(),
                issue: "Default server without explicit server_name".to_string(),
                risk: "May respond to requests for unintended hostnames".to_string(),
                fix: "Add explicit server_name directive or use server_name _;".to_string(),
            });
        }

        // Check 2: Unencrypted sensitive paths
        for location in &server.locations {
            if is_sensitive_path(&location.path) && !server.has_ssl() {
                issues.push(SecurityIssue {
                    severity: Severity::Critical,
                    server: server_name.clone(),
                    category: "SSL/TLS".to_string(),
                    issue: format!("Sensitive path '{}' served over HTTP", location.path),
                    risk: "Credentials or sensitive data may be transmitted in plaintext"
                        .to_string(),
                    fix: "Enable SSL for this server or redirect to HTTPS".to_string(),
                });
            }
        }

        // Check 3: Server tokens
        check_server_tokens(&server_name, &mut issues);
    }

    // Filter by severity level
    let min_severity = match level.to_lowercase().as_str() {
        "critical" => Severity::Critical,
        "warning" => Severity::Warning,
        _ => Severity::Info,
    };

    issues.retain(|i| {
        matches!(
            (&i.severity, &min_severity),
            (Severity::Critical, _)
                | (Severity::Warning, Severity::Warning)
                | (Severity::Warning, Severity::Info)
                | (Severity::Info, Severity::Info)
        )
    });

    format_security_analysis(&issues, format, show_fix)
}

#[derive(Debug)]
struct SecurityIssue {
    severity: Severity,
    server: String,
    category: String,
    issue: String,
    risk: String,
    fix: String,
}

fn is_sensitive_path(path: &str) -> bool {
    let sensitive = ["/admin", "/login", "/api", "/auth", "/dashboard"];
    sensitive.iter().any(|p| path.starts_with(p))
}

fn check_server_tokens(server_name: &str, issues: &mut Vec<SecurityIssue>) {
    // Placeholder - would need to check server_tokens directive
    issues.push(SecurityIssue {
        severity: Severity::Info,
        server: server_name.to_string(),
        category: "Information Disclosure".to_string(),
        issue: "Server tokens directive not checked".to_string(),
        risk: "NGINX version may be disclosed in headers".to_string(),
        fix: "Add 'server_tokens off;' in http or server block".to_string(),
    });
}

fn format_security_analysis(
    issues: &[SecurityIssue],
    format: &OutputFormat,
    show_fix: bool,
) -> Result<String> {
    match format {
        OutputFormat::Table => {
            let mut output = String::new();

            output.push_str(&format!("{}\n\n", "=== Security Analysis ===".bold()));

            if issues.is_empty() {
                output.push_str(&format!("{}\n", "✓ No security issues found".green()));
            } else {
                let critical: Vec<_> = issues
                    .iter()
                    .filter(|i| i.severity == Severity::Critical)
                    .collect();
                let warnings: Vec<_> = issues
                    .iter()
                    .filter(|i| i.severity == Severity::Warning)
                    .collect();
                let info: Vec<_> = issues
                    .iter()
                    .filter(|i| i.severity == Severity::Info)
                    .collect();

                if !critical.is_empty() {
                    output.push_str(&format!("{}\n", "CRITICAL:".red().bold()));
                    for issue in &critical {
                        format_security_issue(&mut output, issue, show_fix);
                    }
                }

                if !warnings.is_empty() {
                    output.push_str(&format!("\n{}\n", "WARNINGS:".yellow().bold()));
                    for issue in &warnings {
                        format_security_issue(&mut output, issue, show_fix);
                    }
                }

                if !info.is_empty() {
                    output.push_str(&format!("\n{}\n", "INFORMATION:".blue().bold()));
                    for issue in &info {
                        format_security_issue(&mut output, issue, show_fix);
                    }
                }

                output.push_str(&format!(
                    "\n{}\n  {} critical, {} warnings, {} info\n",
                    "Summary:".bold(),
                    critical.len(),
                    warnings.len(),
                    info.len()
                ));
            }

            Ok(output)
        }
        OutputFormat::Json => {
            let data = serde_json::json!({
                "issues": issues.iter().map(|i| {
                    serde_json::json!({
                        "severity": format!("{:?}", i.severity),
                        "server": i.server,
                        "category": i.category,
                        "issue": i.issue,
                        "risk": i.risk,
                        "fix": i.fix,
                    })
                }).collect::<Vec<_>>()
            });
            serde_json::to_string_pretty(&data).context("Failed to serialize")
        }
        _ => Ok("Format not yet implemented for security analysis".to_string()),
    }
}

fn format_security_issue(output: &mut String, issue: &SecurityIssue, show_fix: bool) {
    output.push_str(&format!(
        "\n  {} {} [{}]\n",
        match issue.severity {
            Severity::Critical => "✗".red(),
            Severity::Warning => "⚠".yellow(),
            Severity::Info => "ℹ".blue(),
        },
        issue.server.bold(),
        issue.category.dimmed()
    ));
    output.push_str(&format!("    Issue: {}\n", issue.issue));
    output.push_str(&format!("    Risk: {}\n", issue.risk.dimmed()));

    if show_fix {
        output.push_str(&format!("    Fix: {}\n", issue.fix.green()));
    }
}
