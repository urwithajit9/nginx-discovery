//! NGINX listen directive representation
//!
//! This module provides types for representing NGINX `listen` directives,
//! including address, port, SSL configuration, and various options like
//! HTTP/2, HTTP/3, `default_server`, and reuseport.

// src/types/listen.rs
/// Represents an NGINX listen directive
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::struct_excessive_bools)]
pub struct ListenDirective {
    /// Address (can be IP or hostname)
    pub address: String,

    /// Port number
    pub port: u16,

    /// SSL enabled
    pub ssl: bool,

    /// HTTP/2 enabled
    pub http2: bool,

    /// HTTP/3 enabled (QUIC)
    pub http3: bool,

    /// Default server
    pub default_server: bool,

    /// Reuse port (`SO_REUSEPORT`)
    pub reuseport: bool,

    /// Backlog size
    pub backlog: Option<u32>,
}

impl ListenDirective {
    /// Create a new listen directive with defaults
    pub fn new(address: impl Into<String>, port: u16) -> Self {
        Self {
            address: address.into(),
            port,
            ssl: false,
            http2: false,
            http3: false,
            default_server: false,
            reuseport: false,
            backlog: None,
        }
    }

    /// Parse from NGINX listen directive arguments
    #[must_use]
    pub fn from_args(args: &[String]) -> Option<Self> {
        if args.is_empty() {
            return None;
        }

        // Parse first argument (address:port or just port)
        let (address, port) = parse_listen_address(&args[0])?;

        let mut directive = Self::new(address, port);

        // Parse options
        for arg in &args[1..] {
            match arg.as_str() {
                "ssl" => directive.ssl = true,
                "http2" => directive.http2 = true,
                "http3" => directive.http3 = true,
                "default_server" | "default" => directive.default_server = true,
                "reuseport" => directive.reuseport = true,
                _ if arg.starts_with("backlog=") => {
                    if let Some(val) = arg.strip_prefix("backlog=") {
                        directive.backlog = val.parse().ok();
                    }
                }
                _ => {} // Ignore unknown options
            }
        }

        Some(directive)
    }
}

/// Parse listen address and port
#[allow(clippy::unnecessary_wraps)]
fn parse_listen_address(addr: &str) -> Option<(String, u16)> {
    // Examples:
    // "80" -> ("*", 80)
    // "0.0.0.0:80" -> ("0.0.0.0", 80)
    // "localhost:8080" -> ("localhost", 8080)
    // "[::]:80" -> ("::", 80)

    if let Ok(port) = addr.parse::<u16>() {
        // Just a port number
        return Some(("*".to_string(), port));
    }

    // Check for [IPv6]:port format
    if addr.starts_with('[') {
        if let Some(bracket_end) = addr.find(']') {
            let ipv6 = &addr[1..bracket_end];
            let port_part = &addr[bracket_end + 1..];
            if let Some(port_str) = port_part.strip_prefix(':') {
                if let Ok(port) = port_str.parse() {
                    return Some((ipv6.to_string(), port));
                }
            }
        }
    }

    // Check for address:port format
    if let Some(colon_pos) = addr.rfind(':') {
        let address = &addr[..colon_pos];
        let port_str = &addr[colon_pos + 1..];
        if let Ok(port) = port_str.parse() {
            return Some((address.to_string(), port));
        }
    }

    // Default port 80 if no port specified
    Some((addr.to_string(), 80))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_listen_new() {
        let listen = ListenDirective::new("0.0.0.0", 80);
        assert_eq!(listen.address, "0.0.0.0");
        assert_eq!(listen.port, 80);
        assert!(!listen.ssl);
        assert!(!listen.http2);
        assert!(!listen.http3);
        assert!(!listen.default_server);
        assert!(!listen.reuseport);
        assert_eq!(listen.backlog, None);
    }

    #[test]
    fn test_from_args_just_port() {
        let args = vec!["80".to_string()];
        let listen = ListenDirective::from_args(&args).unwrap();

        assert_eq!(listen.address, "*");
        assert_eq!(listen.port, 80);
        assert!(!listen.ssl);
    }

    #[test]
    fn test_from_args_address_and_port() {
        let args = vec!["0.0.0.0:8080".to_string()];
        let listen = ListenDirective::from_args(&args).unwrap();

        assert_eq!(listen.address, "0.0.0.0");
        assert_eq!(listen.port, 8080);
    }

    #[test]
    fn test_from_args_with_ssl() {
        let args = vec!["443".to_string(), "ssl".to_string()];
        let listen = ListenDirective::from_args(&args).unwrap();

        assert_eq!(listen.port, 443);
        assert!(listen.ssl);
    }

    #[test]
    fn test_from_args_with_http2() {
        let args = vec!["443".to_string(), "ssl".to_string(), "http2".to_string()];
        let listen = ListenDirective::from_args(&args).unwrap();

        assert!(listen.ssl);
        assert!(listen.http2);
    }

    #[test]
    fn test_from_args_with_http3() {
        let args = vec!["443".to_string(), "http3".to_string()];
        let listen = ListenDirective::from_args(&args).unwrap();

        assert!(listen.http3);
    }

    #[test]
    fn test_from_args_default_server() {
        let args = vec!["80".to_string(), "default_server".to_string()];
        let listen = ListenDirective::from_args(&args).unwrap();

        assert!(listen.default_server);
    }

    #[test]
    fn test_from_args_default_alias() {
        let args = vec!["80".to_string(), "default".to_string()];
        let listen = ListenDirective::from_args(&args).unwrap();

        assert!(listen.default_server);
    }

    #[test]
    fn test_from_args_reuseport() {
        let args = vec!["80".to_string(), "reuseport".to_string()];
        let listen = ListenDirective::from_args(&args).unwrap();

        assert!(listen.reuseport);
    }

    #[test]
    fn test_from_args_with_backlog() {
        let args = vec!["80".to_string(), "backlog=511".to_string()];
        let listen = ListenDirective::from_args(&args).unwrap();

        assert_eq!(listen.backlog, Some(511));
    }

    #[test]
    fn test_from_args_all_options() {
        let args = vec![
            "443".to_string(),
            "ssl".to_string(),
            "http2".to_string(),
            "default_server".to_string(),
            "reuseport".to_string(),
            "backlog=1024".to_string(),
        ];
        let listen = ListenDirective::from_args(&args).unwrap();

        assert_eq!(listen.port, 443);
        assert!(listen.ssl);
        assert!(listen.http2);
        assert!(listen.default_server);
        assert!(listen.reuseport);
        assert_eq!(listen.backlog, Some(1024));
    }

    #[test]
    fn test_from_args_empty() {
        let args: Vec<String> = vec![];
        let listen = ListenDirective::from_args(&args);

        assert!(listen.is_none());
    }

    #[test]
    fn test_parse_listen_address_just_port() {
        let (addr, port) = parse_listen_address("8080").unwrap();
        assert_eq!(addr, "*");
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_parse_listen_address_ipv4_with_port() {
        let (addr, port) = parse_listen_address("192.168.1.1:9000").unwrap();
        assert_eq!(addr, "192.168.1.1");
        assert_eq!(port, 9000);
    }

    #[test]
    fn test_parse_listen_address_localhost() {
        let (addr, port) = parse_listen_address("localhost:3000").unwrap();
        assert_eq!(addr, "localhost");
        assert_eq!(port, 3000);
    }

    #[test]
    fn test_parse_listen_address_ipv6_with_port() {
        let (addr, port) = parse_listen_address("[::1]:8080").unwrap();
        assert_eq!(addr, "::1");
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_parse_listen_address_ipv6_wildcard() {
        let (addr, port) = parse_listen_address("[::]:80").unwrap();
        assert_eq!(addr, "::");
        assert_eq!(port, 80);
    }

    #[test]
    fn test_parse_listen_address_no_port_defaults_to_80() {
        let (addr, port) = parse_listen_address("example.com").unwrap();
        assert_eq!(addr, "example.com");
        assert_eq!(port, 80);
    }

    #[test]
    fn test_parse_listen_address_unix_socket_fallback() {
        // If someone passes something that doesn't parse, should default to port 80
        let (addr, port) = parse_listen_address("unix:/var/run/nginx.sock").unwrap();
        assert_eq!(addr, "unix:/var/run/nginx.sock");
        assert_eq!(port, 80);
    }
}
