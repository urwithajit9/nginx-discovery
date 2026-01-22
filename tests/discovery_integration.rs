//! Integration tests for NginxDiscovery API

use nginx_discovery::prelude::*;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_discovery_from_text_basic() {
    let config = r"
        user nginx;
        worker_processes auto;
    ";

    let discovery = NginxDiscovery::from_config_text(config).unwrap();
    assert_eq!(discovery.config().directives.len(), 2);
}

#[test]
fn test_discovery_access_logs() {
    let config = r"
        http {
            access_log /var/log/nginx/http.log;

            server {
                server_name example.com;
                access_log /var/log/nginx/example.log;

                location / {
                    access_log /var/log/nginx/location.log;
                }
            }
        }
    ";

    let discovery = NginxDiscovery::from_config_text(config).unwrap();
    let logs = discovery.access_logs();

    assert_eq!(logs.len(), 3);

    // Check contexts
    assert!(logs.iter().any(|l| matches!(l.context, LogContext::Main)));
    assert!(logs
        .iter()
        .any(|l| matches!(l.context, LogContext::Server(_))));
    assert!(logs
        .iter()
        .any(|l| matches!(l.context, LogContext::Location(_))));
}

#[test]
fn test_discovery_log_formats() {
    let config = r"
        log_format combined '$remote_addr - $remote_user [$time_local]';
        log_format main '$request $status $body_bytes_sent';
        log_format custom '$host $request_time';
    ";

    let discovery = NginxDiscovery::from_config_text(config).unwrap();
    let formats = discovery.log_formats();

    assert_eq!(formats.len(), 3);
    assert!(formats.iter().any(|f| f.name() == "combined"));
    assert!(formats.iter().any(|f| f.name() == "main"));
    assert!(formats.iter().any(|f| f.name() == "custom"));

    // Check variable extraction
    let combined = formats.iter().find(|f| f.name() == "combined").unwrap();
    assert!(combined.variables().contains(&"remote_addr".to_string()));
    assert!(combined.variables().contains(&"remote_user".to_string()));
    assert!(combined.variables().contains(&"time_local".to_string()));
}

#[test]
fn test_discovery_all_log_files() {
    let config = r"
        access_log /var/log/nginx/access.log;
        access_log /var/log/nginx/access.log;
        access_log /var/log/nginx/other.log;
    ";

    let discovery = NginxDiscovery::from_config_text(config).unwrap();
    let files = discovery.all_log_files();

    // Should deduplicate
    assert_eq!(files.len(), 2);
}

#[test]
fn test_discovery_server_names() {
    let config = r"
        server {
            server_name example.com www.example.com;
        }

        server {
            server_name test.com;
        }

        server {
            server_name api.example.com;
        }
    ";

    let discovery = NginxDiscovery::from_config_text(config).unwrap();
    let names = discovery.server_names();

    assert_eq!(names.len(), 4);
    assert!(names.contains(&"example.com".to_string()));
    assert!(names.contains(&"www.example.com".to_string()));
    assert!(names.contains(&"test.com".to_string()));
    assert!(names.contains(&"api.example.com".to_string()));
}

#[test]
fn test_discovery_from_file() {
    // Create a temporary config file
    let temp_file = NamedTempFile::new().unwrap();
    let config = b"user nginx;\nworker_processes 4;";
    fs::write(temp_file.path(), config).unwrap();

    let discovery = NginxDiscovery::from_config_file(temp_file.path()).unwrap();
    assert_eq!(discovery.config().directives.len(), 2);
    assert!(discovery.config_path().is_some());
    assert_eq!(discovery.config_path().unwrap(), temp_file.path());
}

#[test]
fn test_discovery_summary() {
    let config = r"
        user nginx;
        worker_processes auto;

        log_format combined '$remote_addr';

        server {
            listen 80;
            server_name example.com;
            access_log /var/log/nginx/access.log;
        }
    ";

    let discovery = NginxDiscovery::from_config_text(config).unwrap();
    let summary = discovery.summary();

    assert!(summary.contains("directives"));
    assert!(summary.contains("Server blocks: 1"));
    assert!(summary.contains("Access logs: 1"));
    assert!(summary.contains("Log formats: 1"));
}

#[test]
#[cfg(feature = "serde")]
fn test_discovery_to_json() {
    let config = "user nginx;";
    let discovery = NginxDiscovery::from_config_text(config).unwrap();
    let json = discovery.to_json().unwrap();

    assert!(json.contains("user"));
    assert!(json.contains("nginx"));

    // Should be valid JSON
    let _: serde_json::Value = serde_json::from_str(&json).unwrap();
}

#[test]
#[cfg(feature = "serde")]
fn test_discovery_to_yaml() {
    let config = "user nginx;";
    let discovery = NginxDiscovery::from_config_text(config).unwrap();
    let yaml = discovery.to_yaml().unwrap();

    assert!(yaml.contains("user"));
    assert!(yaml.contains("nginx"));

    // Should be valid YAML
    let _: serde_yaml::Value = serde_yaml::from_str(&yaml).unwrap();
}

#[test]
fn test_discovery_complex_config() {
    let config = r#"
        user nginx;
        worker_processes auto;
        error_log /var/log/nginx/error.log warn;
        pid /var/run/nginx.pid;

        events {
            worker_connections 1024;
        }

        http {
            include /etc/nginx/mime.types;
            default_type application/octet-stream;

            log_format main '$remote_addr - $remote_user [$time_local] "$request" '
                            '$status $body_bytes_sent "$http_referer" '
                            '"$http_user_agent" "$http_x_forwarded_for"';

            access_log /var/log/nginx/access.log main;

            sendfile on;
            keepalive_timeout 65;

            server {
                listen 80;
                server_name example.com www.example.com;

                location / {
                    root /var/www/html;
                    index index.html index.htm;
                }

                location /api {
                    proxy_pass http://backend:8080;
                    access_log /var/log/nginx/api.log main;
                }

                error_page 404 /404.html;
                error_page 500 502 503 504 /50x.html;
            }

            server {
                listen 443 ssl;
                server_name secure.example.com;

                ssl_certificate /etc/nginx/ssl/cert.pem;
                ssl_certificate_key /etc/nginx/ssl/key.pem;

                access_log /var/log/nginx/secure.log main;
            }
        }
    "#;

    let discovery = NginxDiscovery::from_config_text(config).unwrap();

    // Check overall structure
    assert!(!discovery.config().directives.is_empty());

    // Check logs
    let logs = discovery.access_logs();
    assert!(logs.len() >= 3); // http, api, secure

    // Check formats
    let formats = discovery.log_formats();
    assert_eq!(formats.len(), 1);
    assert_eq!(formats[0].name(), "main");

    // Check server names
    let names = discovery.server_names();
    assert!(names.contains(&"example.com".to_string()));
    assert!(names.contains(&"www.example.com".to_string()));
    assert!(names.contains(&"secure.example.com".to_string()));

    // Check summary
    let summary = discovery.summary();
    assert!(summary.contains("Server blocks: 2"));
}

#[test]
fn test_discovery_empty_config() {
    let config = "";
    let discovery = NginxDiscovery::from_config_text(config).unwrap();

    assert_eq!(discovery.config().directives.len(), 0);
    assert_eq!(discovery.access_logs().len(), 0);
    assert_eq!(discovery.log_formats().len(), 0);
    assert_eq!(discovery.server_names().len(), 0);
}

#[test]
fn test_discovery_config_with_comments() {
    let config = r"
        # Main configuration
        user nginx;

        # Worker configuration
        worker_processes auto;

        # HTTP block
        http {
            # Logging
            access_log /var/log/nginx/access.log;
        }
    ";

    let discovery = NginxDiscovery::from_config_text(config).unwrap();

    // Comments should be ignored
    assert_eq!(discovery.config().directives.len(), 3);
    assert_eq!(discovery.access_logs().len(), 1);
}
