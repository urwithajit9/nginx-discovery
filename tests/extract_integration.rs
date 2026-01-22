//! Integration tests for extractors

use nginx_discovery::{extract, parse};

#[test]
fn test_extract_from_real_config() {
    let config = r#"
user nginx;
worker_processes auto;

events {
    worker_connections 1024;
}

http {
    log_format combined '$remote_addr - $remote_user [$time_local] '
                        '"$request" $status $body_bytes_sent '
                        '"$http_referer" "$http_user_agent"';

    access_log /var/log/nginx/access.log combined;

    server {
        server_name example.com;
        listen 80;

        access_log /var/log/nginx/example.log combined buffer=32k;

        location / {
            root /var/www/html;
        }

        location /api {
            access_log /var/log/nginx/api.log;
            proxy_pass http://localhost:3000;
        }
    }

    server {
        server_name test.com;
        access_log /var/log/nginx/test.log combined;
    }
}
"#;

    let parsed = parse(config).expect("Should parse");

    // Extract formats
    let formats = extract::log_formats(&parsed).expect("Should extract formats");
    assert_eq!(formats.len(), 1);
    assert_eq!(formats[0].name(), "combined");
    assert!(formats[0].variables().contains(&"remote_addr".to_string()));

    // Extract logs
    let logs = extract::access_logs(&parsed).expect("Should extract logs");

    // Debug: print what we found
    for (i, log) in logs.iter().enumerate() {
        println!("Log {}: {} - {:?}", i, log.path.display(), log.context);
    }

    // We should have at least 3 logs
    assert!(
        logs.len() >= 3,
        "Expected at least 3 logs, found {}",
        logs.len()
    );

    // Check contexts
    let main_logs: Vec<_> = logs
        .iter()
        .filter(|l| matches!(l.context, nginx_discovery::types::LogContext::Main))
        .collect();
    assert_eq!(main_logs.len(), 1, "Should have 1 main log");

    // Check options
    let with_buffer: Vec<_> = logs
        .iter()
        .filter(|l| l.options.contains_key("buffer"))
        .collect();
    assert_eq!(with_buffer.len(), 1, "Should have 1 log with buffer option");
}
