//! Abstract Syntax Tree (AST) types for NGINX configurations
//!
//! This module provides the core AST types that represent parsed NGINX configurations.
//!
//! # Examples
//!
//! ```rust
//! use nginx_discovery::ast::*;
//!
//! // Create a simple directive: user nginx;
//! let user_directive = Directive::simple("user", vec!["nginx".to_string()]);
//!
//! // Create a server block
//! let server = Directive::block(
//!     "server",
//!     vec![],
//!     vec![
//!         Directive::simple("listen", vec!["80".to_string()]),
//!         Directive::simple("server_name", vec!["example.com".to_string()]),
//!     ],
//! );
//!
//! // Create a config
//! let config = Config::with_directives(vec![user_directive, server]);
//! ```

mod directive;
mod span;
mod value;

pub use directive::{Directive, DirectiveItem};
pub use span::{Span, Spanned};
pub use value::Value;

/// Root configuration node
///
/// Represents a complete NGINX configuration file or a logical section.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Config {
    /// Top-level directives
    pub directives: Vec<Directive>,
}

impl Config {
    /// Create a new empty configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nginx_discovery::ast::Config;
    ///
    /// let config = Config::new();
    /// assert!(config.directives.is_empty());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            directives: Vec::new(),
        }
    }

    /// Create a configuration with directives
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nginx_discovery::ast::{Config, Directive};
    ///
    /// let directives = vec![
    ///     Directive::simple("user", vec!["nginx".to_string()]),
    /// ];
    /// let config = Config::with_directives(directives);
    /// assert_eq!(config.directives.len(), 1);
    /// ```
    #[must_use]
    pub fn with_directives(directives: Vec<Directive>) -> Self {
        Self { directives }
    }

    /// Add a directive to the configuration
    pub fn add_directive(&mut self, directive: Directive) {
        self.directives.push(directive);
    }

    /// Find all directives with a given name (non-recursive)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nginx_discovery::ast::{Config, Directive};
    ///
    /// let config = Config::with_directives(vec![
    ///     Directive::simple("user", vec!["nginx".to_string()]),
    ///     Directive::simple("worker_processes", vec!["auto".to_string()]),
    /// ]);
    ///
    /// let users = config.find_directives("user");
    /// assert_eq!(users.len(), 1);
    /// ```
    #[must_use]
    pub fn find_directives(&self, name: &str) -> Vec<&Directive> {
        self.directives
            .iter()
            .filter(|d| d.name() == name)
            .collect()
    }

    /// Find all directives with a given name (mutable, non-recursive)
    pub fn find_directives_mut(&mut self, name: &str) -> Vec<&mut Directive> {
        self.directives
            .iter_mut()
            .filter(|d| d.name() == name)
            .collect()
    }

    /// Recursively find all directives with a given name
    ///
    /// This searches through the entire configuration tree, including
    /// directives nested in blocks.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nginx_discovery::ast::{Config, Directive};
    ///
    /// let server = Directive::block(
    ///     "server",
    ///     vec![],
    ///     vec![
    ///         Directive::simple("access_log", vec!["/var/log/nginx/access.log".to_string()]),
    ///     ],
    /// );
    ///
    /// let config = Config::with_directives(vec![
    ///     Directive::simple("access_log", vec!["/var/log/nginx/main.log".to_string()]),
    ///     server,
    /// ]);
    ///
    /// let logs = config.find_directives_recursive("access_log");
    /// assert_eq!(logs.len(), 2); // Found both
    /// ```
    #[must_use]
    pub fn find_directives_recursive(&self, name: &str) -> Vec<&Directive> {
        let mut result = Vec::new();
        self.find_directives_recursive_impl(name, &mut result);
        result
    }

    fn find_directives_recursive_impl<'a>(&'a self, name: &str, result: &mut Vec<&'a Directive>) {
        for directive in &self.directives {
            // Check if current directive matches
            if directive.name() == name {
                result.push(directive);
            }
            // Recursively search in children if this is a block
            if let Some(children) = directive.children() {
                for child in children {
                    Self::find_directive_recursive(child, name, result);
                }
            }
        }
    }

    // Helper to recursively search within a single directive
    fn find_directive_recursive<'a>(
        directive: &'a Directive,
        name: &str,
        result: &mut Vec<&'a Directive>,
    ) {
        if directive.name() == name {
            result.push(directive);
        }
        if let Some(children) = directive.children() {
            for child in children {
                Self::find_directive_recursive(child, name, result);
            }
        }
    }

    /// Count total number of directives (including nested)
    #[must_use]
    pub fn count_directives(&self) -> usize {
        let mut count = self.directives.len();
        for directive in &self.directives {
            if let Some(children) = directive.children() {
                let child_config = Config {
                    directives: children.to_vec(),
                };
                count += child_config.count_directives();
            }
        }
        count
    }

    /// Check if the configuration is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.directives.is_empty()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let config = Config::new();
        assert!(config.directives.is_empty());
        assert!(config.is_empty());
    }

    #[test]
    fn test_config_with_directives() {
        let directives = vec![
            Directive::simple("user", vec!["nginx".to_string()]),
            Directive::simple("worker_processes", vec!["auto".to_string()]),
        ];
        let config = Config::with_directives(directives);
        assert_eq!(config.directives.len(), 2);
        assert!(!config.is_empty());
    }

    #[test]
    fn test_config_add_directive() {
        let mut config = Config::new();
        config.add_directive(Directive::simple("user", vec!["nginx".to_string()]));
        assert_eq!(config.directives.len(), 1);
    }

    #[test]
    fn test_find_directives() {
        let config = Config::with_directives(vec![
            Directive::simple("user", vec!["nginx".to_string()]),
            Directive::simple("worker_processes", vec!["auto".to_string()]),
            Directive::simple("user", vec!["www-data".to_string()]),
        ]);

        let users = config.find_directives("user");
        assert_eq!(users.len(), 2);
    }

    #[test]
    fn test_find_directives_recursive() {
        let location = Directive::block(
            "location",
            vec!["/".to_string()],
            vec![Directive::simple(
                "access_log",
                vec!["/var/log/1.log".to_string()],
            )],
        );

        let server = Directive::block("server", vec![], vec![location]);

        let config = Config::with_directives(vec![
            Directive::simple("access_log", vec!["/var/log/main.log".to_string()]),
            server,
        ]);

        let logs = config.find_directives_recursive("access_log");
        assert_eq!(logs.len(), 2);
    }

    #[test]
    fn test_count_directives() {
        let location = Directive::block(
            "location",
            vec!["/".to_string()],
            vec![Directive::simple(
                "proxy_pass",
                vec!["http://backend".to_string()],
            )],
        );

        let server = Directive::block("server", vec![], vec![location]);

        let config = Config::with_directives(vec![
            Directive::simple("user", vec!["nginx".to_string()]),
            server,
        ]);

        // user + server + location + proxy_pass = 4
        assert_eq!(config.count_directives(), 4);
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.is_empty());
    }
}
