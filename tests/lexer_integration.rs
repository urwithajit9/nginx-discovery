//! Integration tests for the lexer

use nginx_discovery::parser::Lexer;

#[test]
fn test_real_world_config() {
    let config = r#"
user nginx;
worker_processes auto;

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

    server {
        listen 80;
        server_name example.com www.example.com;

        location / {
            root /var/www/html;
            index index.html index.htm;
        }

        location /api {
            proxy_pass http://localhost:3000;
            proxy_set_header Host $host;
        }
    }
}
"#;

    let mut lexer = Lexer::new(config);
    let tokens = lexer.tokenize().expect("Should tokenize successfully");

    // Should have many tokens
    assert!(tokens.len() > 50);

    // Check some specific tokens
    assert_eq!(
        tokens[0].kind,
        nginx_discovery::parser::TokenKind::Word("user".to_string())
    );
    assert_eq!(
        tokens[1].kind,
        nginx_discovery::parser::TokenKind::Word("nginx".to_string())
    );
}

#[test]
fn test_multiline_strings() {
    let config = r#"log_format combined '$remote_addr - $remote_user [$time_local]';"#;

    let mut lexer = Lexer::new(config);
    let tokens = lexer.tokenize().unwrap();

    // Should have: log_format, combined, string, ;, EOF
    assert_eq!(tokens.len(), 5);
}

#[test]
fn test_comments_preserved() {
    let config = r#"
# Main configuration
user nginx;  # Run as nginx user
"#;

    let mut lexer = Lexer::new(config);
    let tokens = lexer.tokenize().unwrap();

    // Should include comment tokens
    let comments: Vec<_> = tokens
        .iter()
        .filter(|t| matches!(t.kind, nginx_discovery::parser::TokenKind::Comment(_)))
        .collect();

    assert_eq!(comments.len(), 2);
}