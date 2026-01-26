//! Extract server-related directives from NGINX configuration

use crate::ast::{Config, Directive};
use crate::error::Result;
use crate::types::{
    AccessLog, ErrorLog, ErrorLogLevel, ListenDirective, Location, LocationModifier, LogContext,
    Server,
};
use std::path::PathBuf;

/// Extract all server blocks from configuration
///
/// # Errors
///
/// This function currently does not return errors but returns `Result`
/// for consistency with other extractors.
///
/// # Examples
///
/// ```
/// use nginx_discovery::{parse, extract};
///
/// let config = r"
/// server {
///     listen 80;
///     server_name example.com;
///     root /var/www/html;
/// }
/// ";
///
/// let parsed = parse(config)?;
/// let servers = extract::servers(&parsed)?;
/// assert_eq!(servers.len(), 1);
/// # Ok::<(), nginx_discovery::Error>(())
/// ```
pub fn servers(config: &Config) -> Result<Vec<Server>> {
    let mut result = Vec::new();

    // Find all server blocks
    for server_directive in config.find_directives_recursive("server") {
        if let Some(server) = parse_server(server_directive) {
            result.push(server);
        }
    }

    Ok(result)
}

/// Parse a single server directive
fn parse_server(directive: &Directive) -> Option<Server> {
    let children = directive.children()?;
    let mut server = Server::new();

    for child in children {
        match child.name() {
            "server_name" => {
                for name in child.args_as_strings() {
                    server = server.with_server_name(name);
                }
            }
            "listen" => {
                if let Some(listen) = ListenDirective::from_args(&child.args_as_strings()) {
                    server = server.with_listen(listen);
                }
            }
            "root" => {
                if let Some(root) = child.first_arg() {
                    server = server.with_root(root);
                }
            }
            "index" => {
                for index_file in child.args_as_strings() {
                    server = server.with_index(index_file);
                }
            }
            "access_log" => {
                if let Some(log) = parse_access_log_in_server(child) {
                    server.access_logs.push(log);
                }
            }
            "error_log" => {
                if let Some(log) = parse_error_log_in_server(child) {
                    server.error_logs.push(log);
                }
            }
            "location" => {
                if let Some(location) = parse_location(child) {
                    server = server.with_location(location);
                }
            }
            _ => {} // Ignore other directives for now
        }
    }

    Some(server)
}

/// Parse location block
fn parse_location(directive: &Directive) -> Option<Location> {
    let args = directive.args_as_strings();
    let (modifier, path) = LocationModifier::from_args(&args);

    let children = directive.children()?;
    let mut location = Location::new(path, modifier);

    for child in children {
        match child.name() {
            "root" => {
                if let Some(root) = child.first_arg() {
                    location.root = Some(PathBuf::from(root));
                }
            }
            "proxy_pass" => {
                if let Some(upstream) = child.first_arg() {
                    location.proxy_pass = Some(upstream);
                }
            }
            "access_log" => {
                if let Some(log) = parse_access_log_in_location(child, &location.path) {
                    location.access_logs.push(log);
                }
            }
            _ => {} // Ignore other directives
        }
    }

    Some(location)
}

/// Parse `access_log` in server context
fn parse_access_log_in_server(directive: &Directive) -> Option<AccessLog> {
    let args = directive.args_as_strings();
    if args.is_empty() {
        return None;
    }

    let path = &args[0];
    if path == "off" {
        return None;
    }

    let mut log =
        AccessLog::new(PathBuf::from(path)).with_context(LogContext::Server("_".to_string()));

    // Second argument might be format name
    if args.len() > 1 && !args[1].contains('=') {
        log = log.with_format(args[1].clone());
    }

    // Parse options
    for arg in args.iter().skip(2) {
        if let Some(idx) = arg.find('=') {
            let key = arg[..idx].to_string();
            let value = arg[idx + 1..].to_string();
            log = log.with_option(key, value);
        }
    }

    Some(log)
}

/// Parse `access_log` in location context
fn parse_access_log_in_location(directive: &Directive, location_path: &str) -> Option<AccessLog> {
    let args = directive.args_as_strings();
    if args.is_empty() {
        return None;
    }

    let path = &args[0];
    if path == "off" {
        return None;
    }

    let mut log = AccessLog::new(PathBuf::from(path))
        .with_context(LogContext::Location(location_path.to_string()));

    // Second argument might be format name
    if args.len() > 1 && !args[1].contains('=') {
        log = log.with_format(args[1].clone());
    }

    // Parse options
    for arg in args.iter().skip(2) {
        if let Some(idx) = arg.find('=') {
            let key = arg[..idx].to_string();
            let value = arg[idx + 1..].to_string();
            log = log.with_option(key, value);
        }
    }

    Some(log)
}

/// Parse `error_log` in server context
fn parse_error_log_in_server(directive: &Directive) -> Option<ErrorLog> {
    let args = directive.args_as_strings();
    if args.is_empty() {
        return None;
    }

    let path = PathBuf::from(&args[0]);
    let level = if args.len() > 1 {
        args[1]
            .parse::<ErrorLogLevel>()
            .unwrap_or(ErrorLogLevel::Error)
    } else {
        ErrorLogLevel::Error
    };

    Some(
        ErrorLog::new(path)
            .with_level(level)
            .with_context(LogContext::Server("_".to_string())),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn test_extract_basic_server() {
        let config = r"
        server {
            listen 80;
            server_name example.com;
            root /var/www/html;
        }
        ";

        let parsed = parse(config).unwrap();
        let servers_list = servers(&parsed).unwrap();

        assert_eq!(servers_list.len(), 1);
        assert_eq!(servers_list[0].server_names.len(), 1);
        assert_eq!(servers_list[0].server_names[0], "example.com");
        assert_eq!(servers_list[0].listen.len(), 1);
        assert_eq!(servers_list[0].listen[0].port, 80);
    }

    #[test]
    fn test_extract_multiple_servers() {
        let config = r"
        server {
            listen 80;
            server_name example.com;
        }
        server {
            listen 443 ssl;
            server_name secure.example.com;
        }
        ";

        let parsed = parse(config).unwrap();
        let servers_list = servers(&parsed).unwrap();

        assert_eq!(servers_list.len(), 2);
        assert_eq!(servers_list[0].listen[0].port, 80);
        assert!(!servers_list[0].listen[0].ssl);
        assert_eq!(servers_list[1].listen[0].port, 443);
        assert!(servers_list[1].listen[0].ssl);
    }

    #[test]
    fn test_extract_server_with_locations() {
        let config = r"
        server {
            listen 80;
            server_name example.com;

            location / {
                root /var/www/html;
            }

            location /api {
                proxy_pass http://backend;
            }
        }
        ";

        let parsed = parse(config).unwrap();
        let servers_list = servers(&parsed).unwrap();

        assert_eq!(servers_list.len(), 1);
        assert_eq!(servers_list[0].locations.len(), 2);

        let loc1 = &servers_list[0].locations[0];
        assert_eq!(loc1.path, "/");
        assert!(loc1.root.is_some());

        let loc2 = &servers_list[0].locations[1];
        assert_eq!(loc2.path, "/api");
        assert!(loc2.is_proxy());
    }

    #[test]
    fn test_extract_server_with_logs() {
        let config = r"
        server {
            listen 80;
            access_log /var/log/nginx/access.log;
            error_log /var/log/nginx/error.log warn;
        }
        ";

        let parsed = parse(config).unwrap();
        let servers_list = servers(&parsed).unwrap();

        assert_eq!(servers_list.len(), 1);
        assert_eq!(servers_list[0].access_logs.len(), 1);
        assert_eq!(servers_list[0].error_logs.len(), 1);
        assert_eq!(servers_list[0].error_logs[0].level, ErrorLogLevel::Warn);
    }

    #[test]
    fn test_parse_location_modifiers() {
        let config = r"
        server {
            location = /exact {
                root /var/www;
            }
            location ^~ /prefix {
                root /var/www;
            }
            location ~ \.php$ {
                root /var/www;
            }
        }
        ";

        let parsed = parse(config).unwrap();
        let servers_list = servers(&parsed).unwrap();

        assert_eq!(servers_list[0].locations.len(), 3);
        assert_eq!(
            servers_list[0].locations[0].modifier,
            LocationModifier::Exact
        );
        assert_eq!(
            servers_list[0].locations[1].modifier,
            LocationModifier::PrefixPriority
        );
        assert_eq!(
            servers_list[0].locations[2].modifier,
            LocationModifier::Regex
        );
    }
}
