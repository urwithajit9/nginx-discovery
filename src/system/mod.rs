//! System interaction utilities for NGINX discovery
//!
//! This module provides functions to interact with the system to:
//! - Find the nginx binary
//! - Execute nginx commands
//! - Parse running configurations

use crate::discovery::NginxDiscovery;
use crate::error::{Error, Result};
use std::path::PathBuf;
use std::process::Command;

/// Find the nginx binary on the system
///
/// Searches for the `nginx` binary in the system PATH.
///
/// # Errors
///
/// Returns an error if:
/// - The nginx binary cannot be found in PATH
/// - Insufficient permissions to access the binary
///
/// # Examples
///
/// ```no_run
/// use nginx_discovery::system::find_nginx;
///
/// let nginx_path = find_nginx()?;
/// println!("Found nginx at: {}", nginx_path.display());
/// # Ok::<(), nginx_discovery::Error>(())
/// ```
pub fn find_nginx() -> Result<PathBuf> {
    which::which("nginx").map_err(|e| {
        Error::System(format!(
            "nginx binary not found in PATH: {e}. \
             Please ensure nginx is installed and accessible."
        ))
    })
}

/// Get the nginx version
///
/// Executes `nginx -v` to retrieve the version information.
///
/// # Errors
///
/// Returns an error if:
/// - nginx cannot be found
/// - nginx -v fails to execute
///
/// # Examples
///
/// ```no_run
/// use nginx_discovery::system::nginx_version;
///
/// let version = nginx_version()?;
/// println!("NGINX version: {}", version);
/// # Ok::<(), nginx_discovery::Error>(())
/// ```
pub fn nginx_version() -> Result<String> {
    let nginx = find_nginx()?;

    let output = Command::new(nginx)
        .arg("-v")
        .output()
        .map_err(|e| Error::System(format!("Failed to execute nginx -v: {e}")))?;

    // nginx -v outputs to stderr
    let version_text = String::from_utf8_lossy(&output.stderr);

    // Extract version string (e.g., "nginx version: nginx/1.18.0")
    let version = version_text.lines().next().unwrap_or("unknown").to_string();

    Ok(version)
}

/// Dump the current nginx configuration
///
/// Executes `nginx -T` to dump the complete running configuration,
/// including all included files.
///
/// # Errors
///
/// Returns an error if:
/// - nginx cannot be found
/// - nginx -T fails to execute
/// - Insufficient permissions (nginx -T usually requires root/sudo)
/// - Configuration has syntax errors
///
/// # Examples
///
/// ```no_run
/// use nginx_discovery::system::dump_config;
///
/// let config_text = dump_config()?;
/// println!("Configuration length: {} bytes", config_text.len());
/// # Ok::<(), nginx_discovery::Error>(())
/// ```
pub fn dump_config() -> Result<String> {
    let nginx = find_nginx()?;

    let output = Command::new(nginx).arg("-T").output().map_err(|e| {
        Error::System(format!(
            "Failed to execute nginx -T: {e}. \
                 You may need to run with sudo or as root."
        ))
    })?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .map_err(|e| Error::System(format!("nginx -T output contains invalid UTF-8: {e}")))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(Error::System(format!(
            "nginx -T failed with status {}: {stderr}\n\
             Common causes:\n\
             - Insufficient permissions (try running with sudo)\n\
             - NGINX is not running\n\
             - Configuration has syntax errors",
            output.status
        )))
    }
}

/// Test nginx configuration syntax
///
/// Executes `nginx -t` to test the configuration for syntax errors.
///
/// # Errors
///
/// Returns an error if:
/// - nginx cannot be found
/// - Configuration has syntax errors
/// - nginx -t fails to execute
///
/// # Examples
///
/// ```no_run
/// use nginx_discovery::system::test_config;
///
/// match test_config() {
///     Ok(msg) => println!("Config is valid: {}", msg),
///     Err(e) => eprintln!("Config has errors: {}", e),
/// }
/// # Ok::<(), nginx_discovery::Error>(())
/// ```
pub fn test_config() -> Result<String> {
    let nginx = find_nginx()?;

    let output = Command::new(nginx)
        .arg("-t")
        .output()
        .map_err(|e| Error::System(format!("Failed to execute nginx -t: {e}")))?;

    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        Ok(stderr.to_string())
    } else {
        Err(Error::System(format!(
            "Configuration test failed:\n{stderr}"
        )))
    }
}

/// Detect and parse the running nginx configuration
///
/// This is a convenience function that:
/// 1. Finds the nginx binary
/// 2. Dumps the configuration with `nginx -T`
/// 3. Parses the configuration into a `NginxDiscovery` instance
///
/// # Errors
///
/// Returns an error if:
/// - nginx cannot be found
/// - nginx -T fails
/// - Configuration cannot be parsed
///
/// # Examples
///
/// ```no_run
/// use nginx_discovery::system::detect_and_parse;
///
/// let discovery = detect_and_parse()?;
/// let logs = discovery.access_logs();
/// println!("Found {} access logs", logs.len());
/// # Ok::<(), nginx_discovery::Error>(())
/// ```
pub fn detect_and_parse() -> Result<NginxDiscovery> {
    let config_text = dump_config()?;
    NginxDiscovery::from_config_text(&config_text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires nginx to be installed"]
    fn test_find_nginx() {
        let result = find_nginx();
        assert!(result.is_ok(), "nginx should be findable if installed");
    }

    #[test]
    #[ignore = "requires nginx to be installed"]
    fn test_nginx_version() {
        let result = nginx_version();
        assert!(result.is_ok());
        if let Ok(version) = result {
            assert!(version.contains("nginx"));
        }
    }

    #[test]
    #[ignore = "requires nginx to be installed and running with proper permissions"]
    fn test_dump_config() {
        let result = dump_config();
        // This might fail if not running with proper permissions
        if let Err(err) = result {
            assert!(err.to_string().contains("sudo") || err.to_string().contains("permission"));
        }
    }

    #[test]
    #[ignore = "requires nginx to be installed"]
    fn test_test_config() {
        let result = test_config();
        // Configuration test might fail if config has errors
        // Just check that the function executes
        let _ = result;
    }

    #[test]
    fn test_error_messages() {
        // Test that error messages are helpful
        let err = Error::System("nginx not found".to_string());
        assert!(err.to_string().contains("nginx not found"));
    }
}
