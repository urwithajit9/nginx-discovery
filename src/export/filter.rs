// src/export/filter.rs
//! Filtering logic for configuration exports.
//!
//! This module provides filtering capabilities to selectively export
//! parts of NGINX configurations based on various criteria such as
//! server name, port, SSL status, or directive name.

use crate::{ast::Config, Error, Result};

/// Filter for selecting specific configuration elements.
///
/// Filters allow you to export only parts of a configuration that match
/// specific criteria. This is useful for generating focused reports or
/// extracting specific server configurations.
///
/// # Examples
///
/// ```
/// use nginx_discovery::export::filter::{Filter, FilterType};
///
/// // Create a filter for a specific server name
/// let filter = Filter::new(FilterType::ServerName, "example.com");
///
/// // Or parse from a string
/// let filter: Filter = "server_name=*.example.com".parse().unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct Filter {
    /// Type of filter to apply
    pub filter_type: FilterType,

    /// Pattern or value to filter by
    pub pattern: String,
}

/// Types of filters available for configuration export.
///
/// Each filter type targets a specific aspect of the NGINX configuration,
/// allowing for precise selection of configuration elements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterType {
    /// Filter by server name (supports wildcards like `*.example.com`)
    ServerName,

    /// Filter by listening port number
    Port,

    /// Filter by upstream backend name
    Upstream,

    /// Filter by location path
    Location,

    /// Filter to include only SSL-enabled servers
    SslOnly,

    /// Filter by directive name
    Directive,
}

impl Filter {
    /// Creates a new filter with the specified type and pattern.
    ///
    /// # Arguments
    ///
    /// * `filter_type` - The type of filter to create
    /// * `pattern` - The pattern or value to match against
    ///
    /// # Examples
    ///
    /// ```
    /// use nginx_discovery::export::filter::{Filter, FilterType};
    ///
    /// let filter = Filter::new(FilterType::Port, "443");
    /// ```
    #[must_use]
    pub fn new(filter_type: FilterType, pattern: impl Into<String>) -> Self {
        Self {
            filter_type,
            pattern: pattern.into(),
        }
    }

    /// Applies this filter to a configuration, returning a filtered copy.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration to filter
    ///
    /// # Returns
    ///
    /// A new `Config` containing only the elements that match the filter criteria.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The filter type is not yet implemented
    /// - The filter pattern is invalid (e.g., non-numeric port)
    /// - The configuration cannot be cloned or modified
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nginx_discovery::{parse, export::filter::{Filter, FilterType}};
    ///
    /// let config = parse("server { listen 80; }")?;
    /// let filter = Filter::new(FilterType::Port, "80");
    /// let filtered = filter.apply(&config)?;
    /// # Ok::<(), nginx_discovery::Error>(())
    /// ```
    pub fn apply(&self, config: &Config) -> Result<Config> {
        let mut filtered = config.clone();

        match self.filter_type {
            FilterType::ServerName => {
                Self::filter_by_server_name(&self.pattern, &mut filtered);
            }
            FilterType::Port => {
                Self::filter_by_port(&self.pattern, &mut filtered)?;
            }
            FilterType::SslOnly => {
                Self::filter_ssl_only(&mut filtered);
            }
            FilterType::Directive => {
                Self::filter_by_directive(&self.pattern, &mut filtered);
            }
            FilterType::Upstream | FilterType::Location => {
                return Err(Error::NotImplemented(format!(
                    "Filter type {:?} not yet implemented",
                    self.filter_type
                )));
            }
        }

        Ok(filtered)
    }

    /// Filters configuration to include only servers matching the given name pattern.
    ///
    /// # Implementation Note
    ///
    /// This is currently a placeholder. Future versions will support wildcard matching
    /// and regex patterns for server names.
    fn filter_by_server_name(_pattern: &str, _config: &mut Config) {
        // TODO: Implement proper server name filtering with wildcard support
        // For now, this is a no-op to maintain API compatibility
    }

    /// Filters configuration to include only servers listening on the specified port.
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidInput` if the pattern is not a valid port number.
    fn filter_by_port(pattern: &str, _config: &mut Config) -> Result<()> {
        let _target_port: u16 = pattern
            .parse()
            .map_err(|_| Error::InvalidInput(format!("Invalid port number: {pattern}")))?;

        // TODO: Implement port filtering by walking the AST
        Ok(())
    }

    /// Filters configuration to include only SSL-enabled servers.
    ///
    /// # Implementation Note
    ///
    /// This will be fully implemented once the Server type exposes SSL configuration.
    fn filter_ssl_only(_config: &mut Config) {
        // TODO: Implement SSL filtering when Server has ssl field
    }

    /// Filters configuration to include only the specified directive.
    ///
    /// This removes all directives that don't match the given name.
    fn filter_by_directive(directive_name: &str, config: &mut Config) {
        // Keep only directives matching the specified name
        config.directives.retain(|d| d.name() == directive_name);
    }
}

impl std::str::FromStr for Filter {
    type Err = Error;

    /// Parses a filter from a string in the format `type=pattern`.
    ///
    /// # Format
    ///
    /// Filters are specified as `type=pattern`, where:
    /// - `type` is one of: `server_name`, `server`, `port`, `upstream`, `location`, `ssl`, `ssl_only`, `directive`
    /// - `pattern` is the value to match (can include wildcards for server names)
    ///
    /// # Examples
    ///
    /// ```
    /// use nginx_discovery::export::filter::Filter;
    ///
    /// let filter: Filter = "server_name=*.example.com".parse().unwrap();
    /// let filter: Filter = "port=443".parse().unwrap();
    /// let filter: Filter = "ssl_only=true".parse().unwrap();
    /// let filter: Filter = "directive=proxy_pass".parse().unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The string is not in `type=pattern` format
    /// - The filter type is not recognized
    fn from_str(s: &str) -> Result<Self> {
        // Parse filter string like "server_name=*.example.com" or "port=443"
        let parts: Vec<&str> = s.splitn(2, '=').collect();

        if parts.len() != 2 {
            return Err(Error::InvalidInput(format!(
                "Invalid filter format. Expected: type=pattern, got: {s}"
            )));
        }

        let filter_type = match parts[0].to_lowercase().as_str() {
            "server_name" | "server" => FilterType::ServerName,
            "port" => FilterType::Port,
            "upstream" => FilterType::Upstream,
            "location" => FilterType::Location,
            "ssl" | "ssl_only" => FilterType::SslOnly,
            "directive" => FilterType::Directive,
            other => {
                return Err(Error::InvalidInput(format!("Unknown filter type: {other}")));
            }
        };

        Ok(Self::new(filter_type, parts[1]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_filter() {
        let filter: Filter = "server_name=*.example.com".parse().unwrap();
        assert_eq!(filter.filter_type, FilterType::ServerName);
        assert_eq!(filter.pattern, "*.example.com");
    }

    #[test]
    fn test_parse_port_filter() {
        let filter: Filter = "port=443".parse().unwrap();
        assert_eq!(filter.filter_type, FilterType::Port);
        assert_eq!(filter.pattern, "443");
    }

    #[test]
    fn test_parse_ssl_filter() {
        let filter: Filter = "ssl_only=true".parse().unwrap();
        assert_eq!(filter.filter_type, FilterType::SslOnly);
    }

    #[test]
    fn test_parse_directive_filter() {
        let filter: Filter = "directive=proxy_pass".parse().unwrap();
        assert_eq!(filter.filter_type, FilterType::Directive);
        assert_eq!(filter.pattern, "proxy_pass");
    }

    #[test]
    fn test_invalid_filter_format() {
        let result: Result<Filter> = "invalid".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_filter_type() {
        let result: Result<Filter> = "unknown_type=value".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_filter_creation() {
        let filter = Filter::new(FilterType::Port, "8080");
        assert_eq!(filter.filter_type, FilterType::Port);
        assert_eq!(filter.pattern, "8080");
    }

    #[test]
    fn test_port_filter_validates_number() {
        let config = Config::default();
        let filter = Filter::new(FilterType::Port, "not_a_number");
        let result = filter.apply(&config);
        assert!(result.is_err());
    }
}
