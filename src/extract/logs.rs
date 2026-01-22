//! Extract log-related directives from NGINX configuration

use crate::ast::{Config, Directive};
use crate::error::Result;
use crate::types::{AccessLog, LogContext, LogFormat};
use std::path::PathBuf;

/// Extract all `log_format` directives
///
/// # Errors
///
/// This function currently does not return errors but returns `Result`
/// for consistency with other extractors.
pub fn log_formats(config: &Config) -> Result<Vec<LogFormat>> {
    let mut formats = Vec::new();

    for directive in config.find_directives_recursive("log_format") {
        if let Some(format) = parse_log_format(directive) {
            formats.push(format);
        }
    }

    Ok(formats)
}

/// Extract all `access_log` directives
///
/// # Errors
///
/// This function currently does not return errors but returns `Result`
/// for consistency with other extractors.
pub fn access_logs(config: &Config) -> Result<Vec<AccessLog>> {
    let mut logs = Vec::new();

    // Find in top-level and http context
    for directive in config.find_directives("access_log") {
        if let Some(log) = parse_access_log(directive, LogContext::Main) {
            logs.push(log);
        }
    }

    // Also check inside http blocks
    for http in config.find_directives("http") {
        for directive in http.find_children("access_log") {
            if let Some(log) = parse_access_log(directive, LogContext::Main) {
                logs.push(log);
            }
        }
    }

    // Find in server blocks
    for server in config.find_directives_recursive("server") {
        let server_name = get_server_name(server);
        let context = LogContext::Server(server_name);

        for directive in server.find_children("access_log") {
            if let Some(log) = parse_access_log(directive, context.clone()) {
                logs.push(log);
            }
        }

        // Find in location blocks within server
        for location in server.find_recursive("location") {
            let location_path = location.first_arg().unwrap_or_else(|| "/".to_string());
            let context = LogContext::Location(location_path);

            for directive in location.find_children("access_log") {
                if let Some(log) = parse_access_log(directive, context.clone()) {
                    logs.push(log);
                }
            }
        }
    }

    Ok(logs)
}

/// Parse a `log_format` directive
fn parse_log_format(directive: &Directive) -> Option<LogFormat> {
    let args = directive.args_as_strings();
    if args.len() < 2 {
        return None;
    }

    let name = args[0].clone();
    let pattern = args[1..].join(" ");

    Some(LogFormat::new(name, pattern))
}

/// Parse an `access_log` directive
fn parse_access_log(directive: &Directive, context: LogContext) -> Option<AccessLog> {
    let args = directive.args_as_strings();
    if args.is_empty() {
        return None;
    }

    let path = &args[0];

    // Skip "off" logs
    if path == "off" {
        return None;
    }

    let mut log = AccessLog::new(PathBuf::from(path)).with_context(context);

    // Second argument might be format name
    if args.len() > 1 {
        let second_arg = &args[1];
        // Check if it's not an option (options contain '=')
        if !second_arg.contains('=') {
            log = log.with_format(second_arg.clone());
        }
    }

    // Parse additional options (buffer=32k, gzip, etc.)
    for arg in args.iter().skip(2) {
        if let Some(idx) = arg.find('=') {
            let key = arg[..idx].to_string();
            let value = arg[idx + 1..].to_string();
            log = log.with_option(key, value);
        }
    }

    Some(log)
}

/// Get `server_name` from a server directive
fn get_server_name(server: &Directive) -> String {
    server
        .find_children("server_name")
        .first()
        .and_then(|d| d.first_arg())
        .unwrap_or_else(|| "_".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn test_extract_log_formats() {
        let config = r"
log_format combined '$remote_addr - $remote_user [$time_local]';
log_format main '$remote_addr $request';
";

        let parsed = parse(config).unwrap();
        let formats = log_formats(&parsed).unwrap();

        assert_eq!(formats.len(), 2);
        assert_eq!(formats[0].name, "combined");
        assert_eq!(formats[1].name, "main");
    }

    #[test]
    fn test_extract_access_logs() {
        let config = r"
access_log /var/log/nginx/access.log combined;
access_log /var/log/nginx/main.log main buffer=32k;
";

        let parsed = parse(config).unwrap();
        let logs = access_logs(&parsed).unwrap();

        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].path, PathBuf::from("/var/log/nginx/access.log"));
        assert_eq!(logs[0].format_name, Some("combined".to_string()));
        assert_eq!(logs[1].format_name, Some("main".to_string()));
        assert_eq!(logs[1].options.get("buffer"), Some(&"32k".to_string()));
    }

    #[test]
    fn test_extract_logs_from_server() {
        let config = r"
server {
    server_name example.com;
    access_log /var/log/nginx/example.log;
}
";

        let parsed = parse(config).unwrap();
        let logs = access_logs(&parsed).unwrap();

        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].path, PathBuf::from("/var/log/nginx/example.log"));
        assert!(matches!(logs[0].context, LogContext::Server(_)));
    }

    #[test]
    fn test_skip_disabled_logs() {
        let config = r"
access_log off;
access_log /var/log/nginx/access.log;
";

        let parsed = parse(config).unwrap();
        let logs = access_logs(&parsed).unwrap();

        // Should only have one log (the enabled one)
        assert_eq!(logs.len(), 1);
    }

    #[test]
    fn test_extract_logs_from_location() {
        let config = r"
server {
    location /api {
        access_log /var/log/nginx/api.log;
    }
}
";

        let parsed = parse(config).unwrap();
        let logs = access_logs(&parsed).unwrap();

        assert_eq!(logs.len(), 1);
        assert!(matches!(logs[0].context, LogContext::Location(_)));
    }
}
