//! NGINX server block representation
//!
//! This module provides types for representing NGINX `server` blocks,
//! including listen directives, server names, locations, and associated logs.

// src/types/server.rs
use crate::types::{AccessLog, ErrorLog, ListenDirective, Location};
use std::path::PathBuf;
// ... rest of file

/// Represents an NGINX server block
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Server {
    /// Server names (from `server_name` directive)
    pub server_names: Vec<String>,

    /// Listen directives
    pub listen: Vec<ListenDirective>,

    /// Root directory
    pub root: Option<PathBuf>,

    /// Location blocks
    pub locations: Vec<Location>,

    /// Access logs specific to this server
    pub access_logs: Vec<AccessLog>,

    /// Error logs specific to this server
    pub error_logs: Vec<ErrorLog>,

    /// Index files
    pub index: Vec<String>,
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Server {
    /// Create a new server
    #[must_use]
    pub fn new() -> Self {
        Self {
            server_names: Vec::new(),
            listen: Vec::new(),
            root: None,
            locations: Vec::new(),
            access_logs: Vec::new(),
            error_logs: Vec::new(),
            index: Vec::new(),
        }
    }

    /// Add a server name
    #[must_use]
    pub fn with_server_name(mut self, name: impl Into<String>) -> Self {
        self.server_names.push(name.into());
        self
    }

    /// Add a listen directive
    #[must_use]
    pub fn with_listen(mut self, listen: ListenDirective) -> Self {
        self.listen.push(listen);
        self
    }

    /// Set root directory
    #[must_use]
    pub fn with_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root = Some(root.into());
        self
    }

    /// Check if server has SSL enabled
    #[must_use]
    pub fn has_ssl(&self) -> bool {
        self.listen.iter().any(|l| l.ssl)
    }

    /// Check if server is default
    #[must_use]
    pub fn is_default_server(&self) -> bool {
        self.listen.iter().any(|l| l.default_server)
    }

    /// Get primary server name
    #[must_use]
    pub fn primary_name(&self) -> Option<&str> {
        self.server_names.first().map(String::as_str)
    }

    /// Add an index file
    #[must_use]
    pub fn with_index(mut self, index: impl Into<String>) -> Self {
        self.index.push(index.into());
        self
    }

    /// Add a location block
    #[must_use]
    pub fn with_location(mut self, location: Location) -> Self {
        self.locations.push(location);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::LocationModifier;

    #[test]
    fn test_server_new() {
        let server = Server::new();
        assert!(server.server_names.is_empty());
        assert!(server.listen.is_empty());
        assert!(server.root.is_none());
        assert!(server.locations.is_empty());
        assert!(server.access_logs.is_empty());
        assert!(server.error_logs.is_empty());
        assert!(server.index.is_empty());
    }

    #[test]
    fn test_server_default() {
        let server = Server::default();
        assert!(server.server_names.is_empty());
        assert!(server.listen.is_empty());
    }

    #[test]
    fn test_with_server_name() {
        let server = Server::new()
            .with_server_name("example.com")
            .with_server_name("www.example.com");

        assert_eq!(server.server_names.len(), 2);
        assert_eq!(server.server_names[0], "example.com");
        assert_eq!(server.server_names[1], "www.example.com");
    }

    #[test]
    fn test_with_listen() {
        let listen = ListenDirective::new("0.0.0.0", 80);
        let server = Server::new().with_listen(listen.clone());

        assert_eq!(server.listen.len(), 1);
        assert_eq!(server.listen[0].port, 80);
        assert_eq!(server.listen[0].address, "0.0.0.0");
    }

    #[test]
    fn test_with_root() {
        let server = Server::new().with_root("/var/www/html");

        assert!(server.root.is_some());
        assert_eq!(server.root.unwrap(), PathBuf::from("/var/www/html"));
    }

    #[test]
    fn test_with_index() {
        let server = Server::new()
            .with_index("index.html")
            .with_index("index.php");

        assert_eq!(server.index.len(), 2);
        assert_eq!(server.index[0], "index.html");
        assert_eq!(server.index[1], "index.php");
    }

    #[test]
    fn test_with_location() {
        let location = Location::new("/", LocationModifier::None);
        let server = Server::new().with_location(location);

        assert_eq!(server.locations.len(), 1);
        assert_eq!(server.locations[0].path, "/");
    }

    #[test]
    fn test_has_ssl_true() {
        let mut listen_ssl = ListenDirective::new("0.0.0.0", 443);
        listen_ssl.ssl = true;
        let server = Server::new().with_listen(listen_ssl);

        assert!(server.has_ssl());
    }

    #[test]
    fn test_has_ssl_false() {
        let listen = ListenDirective::new("0.0.0.0", 80);
        let server = Server::new().with_listen(listen);

        assert!(!server.has_ssl());
    }

    #[test]
    fn test_has_ssl_mixed() {
        let listen_http = ListenDirective::new("0.0.0.0", 80);
        let mut listen_https = ListenDirective::new("0.0.0.0", 443);
        listen_https.ssl = true;

        let server = Server::new()
            .with_listen(listen_http)
            .with_listen(listen_https);

        assert!(server.has_ssl());
    }

    #[test]
    fn test_is_default_server_true() {
        let mut listen = ListenDirective::new("0.0.0.0", 80);
        listen.default_server = true;
        let server = Server::new().with_listen(listen);

        assert!(server.is_default_server());
    }

    #[test]
    fn test_is_default_server_false() {
        let listen = ListenDirective::new("0.0.0.0", 80);
        let server = Server::new().with_listen(listen);

        assert!(!server.is_default_server());
    }

    #[test]
    fn test_primary_name_some() {
        let server = Server::new()
            .with_server_name("example.com")
            .with_server_name("www.example.com");

        assert_eq!(server.primary_name(), Some("example.com"));
    }

    #[test]
    fn test_primary_name_none() {
        let server = Server::new();
        assert_eq!(server.primary_name(), None);
    }

    #[test]
    fn test_builder_pattern_complete() {
        let mut listen = ListenDirective::new("0.0.0.0", 443);
        listen.ssl = true;
        let location = Location::new("/api", LocationModifier::None);

        let server = Server::new()
            .with_server_name("example.com")
            .with_listen(listen)
            .with_root("/var/www")
            .with_index("index.html")
            .with_location(location);

        assert_eq!(server.server_names.len(), 1);
        assert_eq!(server.listen.len(), 1);
        assert!(server.root.is_some());
        assert_eq!(server.index.len(), 1);
        assert_eq!(server.locations.len(), 1);
        assert!(server.has_ssl());
    }
}
