// src/export/filter.rs
//! Filtering logic for exports

use crate::{ast::Config, Result, Error};

/// Filter for selecting specific configuration elements
#[derive(Debug, Clone)]
pub struct Filter {
    /// Filter type
    pub filter_type: FilterType,

    /// Filter value/pattern
    pub pattern: String,
}

/// Types of filters available
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterType {
    /// Filter by server name (supports wildcards)
    ServerName,

    /// Filter by port
    Port,

    /// Filter by upstream name
    Upstream,

    /// Filter by location path
    Location,

    /// Filter servers with SSL enabled
    SslOnly,

    /// Filter by directive name
    Directive,
}

impl Filter {
    /// Create a new filter
    pub fn new(filter_type: FilterType, pattern: impl Into<String>) -> Self {
        Self {
            filter_type,
            pattern: pattern.into(),
        }
    }

    /// Apply this filter to a configuration
    pub fn apply(&self, config: &Config) -> Result<Config> {
        let mut filtered = config.clone();

        match self.filter_type {
            FilterType::ServerName => {
                self.filter_by_server_name(&mut filtered)?;
            }
            FilterType::Port => {
                self.filter_by_port(&mut filtered)?;
            }
            FilterType::SslOnly => {
                self.filter_ssl_only(&mut filtered)?;
            }
            FilterType::Directive => {
                self.filter_by_directive(&mut filtered)?;
            }
            _ => {
                return Err(Error::NotImplemented(format!(
                    "Filter type {:?} not yet implemented",
                    self.filter_type
                )));
            }
        }

        Ok(filtered)
    }

    /// Filter servers by name pattern
    fn filter_by_server_name(&self, _config: &mut Config) -> Result<()> {
        // Simplified implementation - just return OK for now
        // TODO: Implement proper server name filtering
        Ok(())
    }

    /// Filter by port number
    fn filter_by_port(&self, _config: &mut Config) -> Result<()> {
        let _target_port: u16 = self.pattern.parse()
            .map_err(|_| Error::InvalidInput(format!("Invalid port number: {}", self.pattern)))?;

        // TODO: Implement port filtering
        Ok(())
    }

    /// Filter to only SSL-enabled servers
    fn filter_ssl_only(&self, _config: &mut Config) -> Result<()> {
        // TODO: Implement SSL filtering when Server has ssl field
        Ok(())
    }

    /// Filter by directive name
    fn filter_by_directive(&self, config: &mut Config) -> Result<()> {
        let directive_name = &self.pattern;

        // Keep only specified directives
        config.directives.retain(|d| d.name() == directive_name);

        Ok(())
    }
}

impl std::str::FromStr for Filter {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        // Parse filter string like "server_name=*.example.com" or "port=443"
        let parts: Vec<&str> = s.splitn(2, '=').collect();

        if parts.len() != 2 {
            return Err(Error::InvalidInput(format!(
                "Invalid filter format. Expected: type=pattern, got: {}", s
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
                return Err(Error::InvalidInput(format!("Unknown filter type: {}", other)));
            }
        };

        Ok(Filter::new(filter_type, parts[1]))
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
    fn test_invalid_filter() {
        let result: Result<Filter> = "invalid".parse();
        assert!(result.is_err());
    }
}