//! Example: Extract log configurations from NGINX config

use nginx_discovery::{extract, parse};

fn main() {
    let config = r#"
# NGINX Configuration
http {
    # Define log formats
    log_format combined '$remote_addr - $remote_user [$time_local] '
                        '"$request" $status $body_bytes_sent '
                        '"$http_referer" "$http_user_agent"';

    log_format main '$remote_addr - $request - $status';

    # Main access log
    access_log /var/log/nginx/access.log combined;

    server {
        server_name example.com;
        listen 80;

        # Server-specific log
        access_log /var/log/nginx/example.log main buffer=32k;

        location /api {
            # Location-specific log
            access_log /var/log/nginx/api.log combined;
            proxy_pass http://localhost:3000;
        }

        location /static {
            # Disabled log
            access_log off;
            root /var/www/static;
        }
    }

    server {
        server_name test.com;
        access_log /var/log/nginx/test.log;
    }
}
"#;

    println!("Parsing NGINX configuration...\n");

    match parse(config) {
        Ok(parsed) => {
            // Extract log formats
            match extract::log_formats(&parsed) {
                Ok(formats) => {
                    println!("📋 Log Formats ({}):", formats.len());
                    for format in formats {
                        println!("  • {}", format.name());
                        println!("    Pattern: {}", format.pattern());
                        println!("    Variables: {}", format.variables().join(", "));
                        println!();
                    }
                }
                Err(e) => eprintln!("Error extracting formats: {}", e),
            }

            // Extract access logs
            match extract::access_logs(&parsed) {
                Ok(logs) => {
                    println!("📝 Access Logs ({}):", logs.len());
                    for log in logs {
                        println!("  • {}", log.path.display());
                        if let Some(format) = &log.format_name {
                            println!("    Format: {}", format);
                        }
                        println!("    Context: {:?}", log.context);
                        if !log.options.is_empty() {
                            println!("    Options: {:?}", log.options);
                        }
                        println!();
                    }
                }
                Err(e) => eprintln!("Error extracting logs: {}", e),
            }
        }
        Err(e) => {
            eprintln!("Parse error:\n{}", e.detailed());
        }
    }
}
