//! Integration tests for the parser

use nginx_discovery::parse;

#[test]
fn test_parse_basic_config() {
    let config = r#"
user nginx;
worker_processes auto;
error_log /var/log/nginx/error.log;
"#;

    let result = parse(config).unwrap();
    assert_eq!(result.directives.len(), 3);

    assert_eq!(result.directives[0].name(), "user");
    assert_eq!(result.directives[1].name(), "worker_processes");
    assert_eq!(result.directives[2].name(), "error_log");
}

#[test]
fn test_parse_http_block() {
    let config = r#"
http {
    include /etc/nginx/mime.types;
    default_type application/octet-stream;

    sendfile on;
    keepalive_timeout 65;
}
"#;

    let result = parse(config).unwrap();
    assert_eq!(result.directives.len(), 1);

    let http = &result.directives[0];
    assert_eq!(http.name(), "http");
    assert!(http.is_block());
    assert_eq!(http.children().unwrap().len(), 4);
}

#[test]
fn test_parse_server_with_locations() {
    let config = r#"
server {
    listen 80;
    server_name example.com;
    root /var/www/html;

    location / {
        index index.html;
    }

    location /api {
        proxy_pass http://localhost:3000;
    }
}
"#;

    let result = parse(config).unwrap();
    let server = &result.directives[0];

    assert_eq!(server.name(), "server");
    assert_eq!(server.children().unwrap().len(), 5); // 3 simple + 2 locations
}

#[test]
fn test_parse_log_format() {
    let config = r#"
log_format main '$remote_addr - $remote_user [$time_local] "$request"';
access_log /var/log/nginx/access.log main;
"#;

    let result = parse(config).unwrap();
    assert_eq!(result.directives.len(), 2);

    let log_format = &result.directives[0];
    assert_eq!(log_format.name(), "log_format");
    assert_eq!(log_format.args().len(), 2); // name + format string
}

#[test]
fn test_parse_upstream() {
    let config = r#"
upstream backend {
    server backend1.example.com:8080;
    server backend2.example.com:8080;
    server backend3.example.com:8080;
}
"#;

    let result = parse(config).unwrap();
    let upstream = &result.directives[0];

    assert_eq!(upstream.name(), "upstream");
    assert_eq!(upstream.args().len(), 1); // backend
    assert_eq!(upstream.children().unwrap().len(), 3); // 3 servers
}


#[test]
fn test_parse_full_config() {
    let config = r#"
user nginx;
worker_processes auto;

events {
    worker_connections 1024;
}

http {
    include /etc/nginx/mime.types;

    log_format main '$remote_addr - $remote_user [$time_local]';
    access_log /var/log/nginx/access.log main;

    upstream backend {
        server localhost:8080;  # Changed from 127.0.0.1:8080
    }

    server {
        listen 80;
        server_name example.com;

        location / {
            proxy_pass http://backend;
            proxy_set_header Host $host;
        }
    }
}
"#;

    let result = parse(config).unwrap();
    assert_eq!(result.directives.len(), 4); // user, worker_processes, events, http

    // Verify http block has correct structure
    let http = &result.directives[3];
    assert_eq!(http.name(), "http");
    assert!(http.children().is_some());
}

#[test]
fn test_parse_with_variables() {
    let config = r#"
set $my_var "value";
proxy_set_header Host $host;
"#;

    let result = parse(config).unwrap();
    assert_eq!(result.directives.len(), 2);

    // Check that variables are preserved
    let set_directive = &result.directives[0];
    assert!(set_directive.args()[0].is_variable());
}

#[test]
fn test_parse_comments_ignored() {
    let config = r#"
# This is a comment
user nginx;  # inline comment
# Another comment
worker_processes auto;
"#;

    let result = parse(config).unwrap();
    // Comments should be skipped, only 2 directives
    assert_eq!(result.directives.len(), 2);
}

#[test]
fn test_parse_empty_block() {
    let config = "events { }";
    let result = parse(config).unwrap();

    let events = &result.directives[0];
    assert_eq!(events.name(), "events");
    assert_eq!(events.children().unwrap().len(), 0);
}

#[test]
fn test_parse_error_missing_semicolon() {
    let config = "user nginx"; // Missing semicolon
    let result = parse(config);

    assert!(result.is_err());
}

#[test]
fn test_parse_error_unclosed_block() {
    let config = "server { listen 80;"; // Missing }
    let result = parse(config);

    assert!(result.is_err());
}
