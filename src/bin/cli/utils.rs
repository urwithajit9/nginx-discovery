//! CLI utility functions

use crate::cli::args::{ColorChoice, GlobalOpts};
use anyhow::Result;
use colored::control;
use nginx_discovery::system;
use std::path::PathBuf;

/// Setup color output based on user preference
pub fn setup_colors(choice: ColorChoice) {
    match choice {
        ColorChoice::Always => control::set_override(true),
        ColorChoice::Never => control::set_override(false),
        ColorChoice::Auto => {
            // Auto-detect based on terminal
            if atty::is(atty::Stream::Stdout) {
                control::set_override(true);
            } else {
                control::set_override(false);
            }
        }
    }
}

/// Find NGINX configuration file
pub fn find_config(global: &GlobalOpts) -> Result<PathBuf> {
    if let Some(ref path) = global.config {
        // User specified a path
        return Ok(path.clone());
    }

    // Try to auto-detect
    if global.verbose {
        eprintln!("Auto-detecting NGINX configuration...");
    }

    // Common locations
    let common_paths = [
        "/etc/nginx/nginx.conf",
        "/usr/local/nginx/conf/nginx.conf",
        "/usr/local/etc/nginx/nginx.conf",
        "nginx.conf",
    ];

    for path in &common_paths {
        let p = PathBuf::from(path);
        if p.exists() {
            if global.verbose {
                eprintln!("Found config at: {}", p.display());
            }
            return Ok(p);
        }
    }

    // Try using nginx -V to find config
    if let Ok(_nginx_path) = system::find_nginx() {
        if global.verbose {
            eprintln!("Trying to detect config from nginx binary...");
        }

        // Parse nginx -V output to find --conf-path
        // This is a simplified version - full implementation would parse the output
        let default_path = PathBuf::from("/etc/nginx/nginx.conf");
        if default_path.exists() {
            return Ok(default_path);
        }
    }

    anyhow::bail!("Could not find NGINX configuration file. Please specify with --config")
}

// Simple check if running in a terminal (fallback if atty crate not available)
mod atty {
    pub enum Stream {
        Stdout,
    }

    pub fn is(_: Stream) -> bool {
        // Simple check: if we can get terminal size, we're probably in a terminal
        std::env::var("TERM").is_ok()
    }
}
