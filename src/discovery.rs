//! High-level discovery API for NGINX configurations
//!
//! This module provides a convenient API for discovering and analyzing NGINX configurations.
//!
//! # Examples
//!
//! ## Parse from text
//!
//! ```
//! use nginx_discovery::NginxDiscovery;
//!
//! let config = r"
//! http {
//!     access_log /var/log/nginx/access.log;
//! }
//! ";
//!
//! let discovery = NginxDiscovery::from_config_text(config)?;
//! let logs = discovery.access_logs();
//! assert_eq!(logs.len(), 1);
//! # Ok::<(), nginx_discovery::Error>(())
//! ```
//!
//! ## Parse from file
//!
//! ```no_run
//! use nginx_discovery::NginxDiscovery;
//!
//! let discovery = NginxDiscovery::from_config_file("/etc/nginx/nginx.conf")?;
//! let logs = discovery.access_logs();
//! let formats = discovery.log_formats();
//! # Ok::<(), nginx_discovery::Error>(())
//! ```

use crate::ast::Config;
use crate::error::Result;
use crate::extract;
use crate::types::{AccessLog, LogFormat};
use std::path::{Path, PathBuf};

/// High-level NGINX configuration discovery
///
/// Provides convenient methods to discover and analyze NGINX configurations.
#[derive(Debug, Clone)]
pub struct NginxDiscovery {
    /// Parsed configuration
    config: Config,
    /// Path to the configuration file (if loaded from file)
    config_path: Option<PathBuf>,
}

impl NginxDiscovery {
    /// Create a discovery instance from configuration text
    ///
    /// # Arguments
    ///
    /// * `text` - NGINX configuration as a string
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```
    /// use nginx_discovery::NginxDiscovery;
    ///
    /// let config = "user nginx;";
    /// let discovery = NginxDiscovery::from_config_text(config)?;
    /// # Ok::<(), nginx_discovery::Error>(())
    /// ```
    pub fn from_config_text(text: &str) -> Result<Self> {
        let config = crate::parse(text)?;
        Ok(Self {
            config,
            config_path: None,
        })
    }

    /// Create a discovery instance from a configuration file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the NGINX configuration file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be read
    /// - The configuration cannot be parsed
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nginx_discovery::NginxDiscovery;
    ///
    /// let discovery = NginxDiscovery::from_config_file("/etc/nginx/nginx.conf")?;
    /// # Ok::<(), nginx_discovery::Error>(())
    /// ```
    pub fn from_config_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path)?;
        let config = crate::parse(&text)?;
        Ok(Self {
            config,
            config_path: Some(path.to_path_buf()),
        })
    }

    /// Create a discovery instance from a running NGINX instance
    ///
    /// This attempts to:
    /// 1. Find the nginx binary
    /// 2. Run `nginx -T` to dump the configuration
    /// 3. Parse the dumped configuration
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - nginx binary cannot be found
    /// - nginx -T fails to execute
    /// - The configuration cannot be parsed
    /// - Insufficient permissions to run nginx -T
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nginx_discovery::NginxDiscovery;
    ///
    /// let discovery = NginxDiscovery::from_running_instance()?;
    /// let logs = discovery.access_logs();
    /// # Ok::<(), nginx_discovery::Error>(())
    /// ```
    #[cfg(feature = "system")]
    pub fn from_running_instance() -> Result<Self> {
        crate::system::detect_and_parse()
    }

    /// Get all access log configurations
    ///
    /// Returns all `access_log` directives found in the configuration,
    /// including those in http, server, and location contexts.
    ///
    /// # Examples
    ///
    /// ```
    /// use nginx_discovery::NginxDiscovery;
    ///
    /// let config = r"
    /// http {
    ///     access_log /var/log/nginx/access.log;
    ///     server {
    ///         access_log /var/log/nginx/server.log;
    ///     }
    /// }
    /// ";
    ///
    /// let discovery = NginxDiscovery::from_config_text(config)?;
    /// let logs = discovery.access_logs();
    /// assert_eq!(logs.len(), 2);
    /// # Ok::<(), nginx_discovery::Error>(())
    /// ```
    #[must_use]
    pub fn access_logs(&self) -> Vec<AccessLog> {
        extract::access_logs(&self.config).unwrap_or_default()
    }

    /// Get all log format definitions
    ///
    /// Returns all `log_format` directives found in the configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use nginx_discovery::NginxDiscovery;
    ///
    /// let config = r"
    /// log_format combined '$remote_addr - $remote_user [$time_local]';
    /// log_format main '$request $status';
    /// ";
    ///
    /// let discovery = NginxDiscovery::from_config_text(config)?;
    /// let formats = discovery.log_formats();
    /// assert_eq!(formats.len(), 2);
    /// # Ok::<(), nginx_discovery::Error>(())
    /// ```
    #[must_use]
    pub fn log_formats(&self) -> Vec<LogFormat> {
        extract::log_formats(&self.config).unwrap_or_default()
    }

    /// Get all log file paths (access logs only)
    ///
    /// Returns a deduplicated list of all access log file paths.
    ///
    /// # Examples
    ///
    /// ```
    /// use nginx_discovery::NginxDiscovery;
    ///
    /// let config = r"
    /// access_log /var/log/nginx/access.log;
    /// access_log /var/log/nginx/access.log;  # duplicate
    /// access_log /var/log/nginx/other.log;
    /// ";
    ///
    /// let discovery = NginxDiscovery::from_config_text(config)?;
    /// let files = discovery.all_log_files();
    /// assert_eq!(files.len(), 2); // deduplicated
    /// # Ok::<(), nginx_discovery::Error>(())
    /// ```
    #[must_use]
    pub fn all_log_files(&self) -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = self.access_logs().into_iter().map(|log| log.path).collect();

        // Deduplicate
        paths.sort();
        paths.dedup();
        paths
    }

    /// Get all server names from server blocks
    ///
    /// Returns a list of all server names defined in server blocks.
    ///
    /// # Examples
    ///
    /// ```
    /// use nginx_discovery::NginxDiscovery;
    ///
    /// let config = r"
    /// server {
    ///     server_name example.com www.example.com;
    /// }
    /// server {
    ///     server_name test.com;
    /// }
    /// ";
    ///
    /// let discovery = NginxDiscovery::from_config_text(config)?;
    /// let names = discovery.server_names();
    /// assert_eq!(names.len(), 3);
    /// # Ok::<(), nginx_discovery::Error>(())
    /// ```
    #[must_use]
    pub fn server_names(&self) -> Vec<String> {
        let mut names = Vec::new();

        for server in self.config.find_directives_recursive("server") {
            for server_name_directive in server.find_children("server_name") {
                names.extend(server_name_directive.args_as_strings());
            }
        }

        names
    }

    /// Export configuration to JSON
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use nginx_discovery::NginxDiscovery;
    ///
    /// let config = "user nginx;";
    /// let discovery = NginxDiscovery::from_config_text(config)?;
    /// let json = discovery.to_json()?;
    /// assert!(json.contains("user"));
    /// # Ok::<(), nginx_discovery::Error>(())
    /// ```
    #[cfg(feature = "serde")]
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.config)
            .map_err(|e| crate::Error::Serialization(e.to_string()))
    }

    /// Export configuration to YAML
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use nginx_discovery::NginxDiscovery;
    ///
    /// let config = "user nginx;";
    /// let discovery = NginxDiscovery::from_config_text(config)?;
    /// let yaml = discovery.to_yaml()?;
    /// assert!(yaml.contains("user"));
    /// # Ok::<(), nginx_discovery::Error>(())
    /// ```
    #[cfg(feature = "serde")]
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(&self.config).map_err(|e| crate::Error::Serialization(e.to_string()))
    }

    /// Get the parsed configuration AST
    ///
    /// Provides direct access to the parsed configuration for custom processing.
    ///
    /// # Examples
    ///
    /// ```
    /// use nginx_discovery::NginxDiscovery;
    ///
    /// let config = "user nginx;";
    /// let discovery = NginxDiscovery::from_config_text(config)?;
    /// let ast = discovery.config();
    /// assert_eq!(ast.directives.len(), 1);
    /// # Ok::<(), nginx_discovery::Error>(())
    /// ```
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get the configuration file path (if loaded from file)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nginx_discovery::NginxDiscovery;
    ///
    /// let discovery = NginxDiscovery::from_config_file("/etc/nginx/nginx.conf")?;
    /// assert!(discovery.config_path().is_some());
    /// # Ok::<(), nginx_discovery::Error>(())
    /// ```
    #[must_use]
    pub fn config_path(&self) -> Option<&Path> {
        self.config_path.as_deref()
    }

    /// Generate a summary of the configuration
    ///
    /// Returns a human-readable summary including:
    /// - Number of directives
    /// - Number of server blocks
    /// - Number of access logs
    /// - Number of log formats
    ///
    /// # Examples
    ///
    /// ```
    /// use nginx_discovery::NginxDiscovery;
    ///
    /// let config = r"
    /// user nginx;
    /// access_log /var/log/nginx/access.log;
    /// server { listen 80; }
    /// ";
    ///
    /// let discovery = NginxDiscovery::from_config_text(config)?;
    /// let summary = discovery.summary();
    /// assert!(summary.contains("directives"));
    /// # Ok::<(), nginx_discovery::Error>(())
    /// ```
    #[must_use]
    pub fn summary(&self) -> String {
        let directive_count = self.config.count_directives();
        let server_count = self.config.find_directives_recursive("server").len();
        let access_log_count = self.access_logs().len();
        let format_count = self.log_formats().len();

        format!(
            "NGINX Configuration Summary:\n\
            - Total directives: {directive_count}\n\
            - Server blocks: {server_count}\n\
            - Access logs: {access_log_count}\n\
            - Log formats: {format_count}"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_config_text() {
        let config = "user nginx;";
        let discovery = NginxDiscovery::from_config_text(config).unwrap();
        assert_eq!(discovery.config.directives.len(), 1);
    }

    #[test]
    fn test_access_logs() {
        let config = r"
        http {
            access_log /var/log/nginx/access.log;
            server {
                access_log /var/log/nginx/server.log;
            }
        }
        ";

        let discovery = NginxDiscovery::from_config_text(config).unwrap();
        let logs = discovery.access_logs();
        assert_eq!(logs.len(), 2);
    }

    #[test]
    fn test_log_formats() {
        let config = r"
        log_format combined '$remote_addr';
        log_format main '$request';
        ";

        let discovery = NginxDiscovery::from_config_text(config).unwrap();
        let formats = discovery.log_formats();
        assert_eq!(formats.len(), 2);
    }

    #[test]
    fn test_all_log_files() {
        let config = r"
        access_log /var/log/nginx/access.log;
        access_log /var/log/nginx/access.log;
        access_log /var/log/nginx/other.log;
        ";

        let discovery = NginxDiscovery::from_config_text(config).unwrap();
        let files = discovery.all_log_files();
        assert_eq!(files.len(), 2); // Deduplicated
    }

    #[test]
    fn test_server_names() {
        let config = r"
        server {
            server_name example.com www.example.com;
        }
        server {
            server_name test.com;
        }
        ";

        let discovery = NginxDiscovery::from_config_text(config).unwrap();
        let names = discovery.server_names();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"example.com".to_string()));
        assert!(names.contains(&"www.example.com".to_string()));
        assert!(names.contains(&"test.com".to_string()));
    }

    #[test]
    fn test_summary() {
        let config = r"
        user nginx;
        access_log /var/log/nginx/access.log;
        server { listen 80; }
        ";

        let discovery = NginxDiscovery::from_config_text(config).unwrap();
        let summary = discovery.summary();
        assert!(summary.contains("directives"));
        assert!(summary.contains("Server blocks: 1"));
    }

    #[test]
    fn test_config_access() {
        let config = "user nginx;";
        let discovery = NginxDiscovery::from_config_text(config).unwrap();
        let ast = discovery.config();
        assert_eq!(ast.directives.len(), 1);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_to_json() {
        let config = "user nginx;";
        let discovery = NginxDiscovery::from_config_text(config).unwrap();
        let json = discovery.to_json().unwrap();
        assert!(json.contains("user"));
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_to_yaml() {
        let config = "user nginx;";
        let discovery = NginxDiscovery::from_config_text(config).unwrap();
        let yaml = discovery.to_yaml().unwrap();
        assert!(yaml.contains("user"));
    }
}
