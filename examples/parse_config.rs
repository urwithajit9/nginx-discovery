//! Example: Parsing an NGINX configuration

use nginx_discovery::parse;

fn main() {
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
                    '$status $body_bytes_sent "$http_referer"';

    access_log /var/log/nginx/access.log main;

    server {
        listen 80;
        server_name example.com www.example.com;
        root /var/www/html;

        location / {
            index index.html index.htm;
        }

        location /api {
            proxy_pass http://localhost:3000;
            proxy_set_header Host $host;
        }
    }
}
"#;

    println!("Parsing NGINX configuration...\n");

    match parse(config) {
        Ok(config) => {
            println!("✅ Parse successful!\n");
            println!("Top-level directives: {}", config.directives.len());

            for directive in &config.directives {
                print_directive(directive, 0);
            }

            println!("\n📊 Statistics:");
            println!("  Total directives: {}", config.count_directives());

            // Find all server blocks
            let servers = config.find_directives_recursive("server");
            println!("  Server blocks: {}", servers.len());

            // Find all location blocks
            let locations = config.find_directives_recursive("location");
            println!("  Location blocks: {}", locations.len());

            // Find all access_log directives
            let access_logs = config.find_directives_recursive("access_log");
            println!("  Access logs: {}", access_logs.len());
        }
        Err(e) => {
            eprintln!("❌ Parse error:\n{}", e.detailed());
        }
    }
}

fn print_directive(directive: &nginx_discovery::ast::Directive, indent: usize) {
    let indent_str = "  ".repeat(indent);

    print!("{}{}", indent_str, directive.name());

    // Print arguments
    for arg in directive.args() {
        print!(" {}", arg);
    }

    if directive.is_block() {
        println!(" {{");
        if let Some(children) = directive.children() {
            for child in children {
                print_directive(child, indent + 1);
            }
        }
        println!("{}}}", indent_str);
    } else {
        println!(";");
    }
}
