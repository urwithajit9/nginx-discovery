//! Integration tests for server discovery features

use nginx_discovery::NginxDiscovery;

#[test]
fn test_servers_basic() {
    let config = r#"
        server {
            listen 80;
            server_name example.com;
        }
    "#;

    let discovery = NginxDiscovery::from_config_text(config).unwrap();
    let servers = discovery.servers();

    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].server_names.len(), 1);
    assert_eq!(servers[0].server_names[0], "example.com");
    assert_eq!(servers[0].listen.len(), 1);
    assert_eq!(servers[0].listen[0].port, 80);
}

#[test]
fn test_servers_multiple() {
    let config = r#"
        server {
            listen 80;
            server_name example.com www.example.com;
        }
        server {
            listen 443 ssl;
            server_name secure.example.com;
        }
        server {
            listen 8080;
            server_name api.example.com;
        }
    "#;

    let discovery = NginxDiscovery::from_config_text(config).unwrap();
    let servers = discovery.servers();

    assert_eq!(servers.len(), 3);
    assert_eq!(servers[0].server_names.len(), 2);
    assert_eq!(servers[1].server_names[0], "secure.example.com");
    assert_eq!(servers[2].listen[0].port, 8080);
}

#[test]
fn test_servers_with_locations() {
    let config = r#"
        server {
            listen 80;
            server_name example.com;

            location / {
                root /var/www/html;
            }

            location /api {
                proxy_pass http://backend:3000;
            }
        }
    "#;

    let discovery = NginxDiscovery::from_config_text(config).unwrap();
    let servers = discovery.servers();

    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].locations.len(), 2);
    assert_eq!(servers[0].locations[0].path, "/");
    assert_eq!(servers[0].locations[1].path, "/api");
    assert!(servers[0].locations[1].is_proxy());
}

#[test]
fn test_listening_ports() {
    let config = r#"
        server {
            listen 80;
            listen 8080;
        }
        server {
            listen 443 ssl;
        }
        server {
            listen 80;
            listen 9000;
        }
    "#;

    let discovery = NginxDiscovery::from_config_text(config).unwrap();
    let ports = discovery.listening_ports();

    // Should be deduplicated and sorted
    assert_eq!(ports.len(), 4);
    assert!(ports.contains(&80));
    assert!(ports.contains(&443));
    assert!(ports.contains(&8080));
    assert!(ports.contains(&9000));

    // Verify sorted
    assert_eq!(ports[0], 80);
    assert_eq!(ports[1], 443);
    assert_eq!(ports[2], 8080);
    assert_eq!(ports[3], 9000);
}

#[test]
fn test_ssl_servers() {
    let config = r#"
        server {
            listen 80;
            server_name http.example.com;
        }
        server {
            listen 443 ssl;
            server_name https.example.com;
        }
        server {
            listen 80;
            listen 443 ssl;
            server_name mixed.example.com;
        }
    "#;

    let discovery = NginxDiscovery::from_config_text(config).unwrap();
    let ssl_servers = discovery.ssl_servers();

    assert_eq!(ssl_servers.len(), 2);
    assert_eq!(ssl_servers[0].primary_name(), Some("https.example.com"));
    assert_eq!(ssl_servers[1].primary_name(), Some("mixed.example.com"));
}

#[test]
fn test_ssl_servers_none() {
    let config = r#"
        server {
            listen 80;
            server_name example.com;
        }
    "#;

    let discovery = NginxDiscovery::from_config_text(config).unwrap();
    let ssl_servers = discovery.ssl_servers();

    assert_eq!(ssl_servers.len(), 0);
}

#[test]
fn test_proxy_locations() {
    let config = r#"
        server {
            location / {
                root /var/www;
            }
            location /api {
                proxy_pass http://api-backend;
            }
            location /static {
                root /var/www/static;
            }
            location /graphql {
                proxy_pass http://graphql-backend:4000;
            }
        }
    "#;

    let discovery = NginxDiscovery::from_config_text(config).unwrap();
    let proxies = discovery.proxy_locations();

    assert_eq!(proxies.len(), 2);
    assert_eq!(proxies[0].path, "/api");
    assert_eq!(proxies[1].path, "/graphql");
    assert!(proxies[0].is_proxy());
    assert!(proxies[1].is_proxy());
}

#[test]
fn test_proxy_locations_none() {
    let config = r#"
        server {
            location / {
                root /var/www;
            }
        }
    "#;

    let discovery = NginxDiscovery::from_config_text(config).unwrap();
    let proxies = discovery.proxy_locations();

    assert_eq!(proxies.len(), 0);
}

#[test]
fn test_location_count() {
    let config = r#"
        server {
            location / {}
            location /api {}
        }
        server {
            location / {}
            location /admin {}
            location /static {}
        }
    "#;

    let discovery = NginxDiscovery::from_config_text(config).unwrap();
    let count = discovery.location_count();

    assert_eq!(count, 5);
}

#[test]
fn test_location_count_zero() {
    let config = r#"
        server {
            listen 80;
        }
    "#;

    let discovery = NginxDiscovery::from_config_text(config).unwrap();
    let count = discovery.location_count();

    assert_eq!(count, 0);
}

#[test]
fn test_complex_server_config() {
    let config = r#"
        http {
            server {
                listen 80;
                server_name example.com www.example.com;
                root /var/www/example;
                index index.html index.htm;

                location / {
                    try_files $uri $uri/ =404;
                }

                location /api {
                    proxy_pass http://localhost:3000;
                    proxy_set_header Host $host;
                }

                location ~ \.php$ {
                    fastcgi_pass unix:/var/run/php/php7.4-fpm.sock;
                }
            }

            server {
                listen 443 ssl http2;
                server_name secure.example.com;

                ssl_certificate /etc/ssl/certs/cert.pem;
                ssl_certificate_key /etc/ssl/private/key.pem;

                location / {
                    root /var/www/secure;
                }

                location /admin {
                    proxy_pass http://admin-backend:8080;
                }
            }
        }
    "#;

    let discovery = NginxDiscovery::from_config_text(config).unwrap();

    // Test servers
    let servers = discovery.servers();
    assert_eq!(servers.len(), 2);

    // Test first server
    assert_eq!(servers[0].server_names.len(), 2);
    assert_eq!(servers[0].listen.len(), 1);
    assert_eq!(servers[0].locations.len(), 3);
    assert!(!servers[0].has_ssl());

    // Test second server
    assert_eq!(servers[1].server_names[0], "secure.example.com");
    assert!(servers[1].has_ssl());
    assert_eq!(servers[1].listen[0].http2, true);
    assert_eq!(servers[1].locations.len(), 2);

    // Test SSL servers
    let ssl_servers = discovery.ssl_servers();
    assert_eq!(ssl_servers.len(), 1);
    assert_eq!(ssl_servers[0].primary_name(), Some("secure.example.com"));

    // Test listening ports
    let ports = discovery.listening_ports();
    assert!(ports.contains(&80));
    assert!(ports.contains(&443));

    // Test proxy locations
    let proxies = discovery.proxy_locations();
    assert_eq!(proxies.len(), 2);

    // Test location count
    let location_count = discovery.location_count();
    assert_eq!(location_count, 5);
}

#[test]
fn test_default_server_detection() {
    let config = r#"
        server {
            listen 80 default_server;
            server_name _;
        }
        server {
            listen 80;
            server_name example.com;
        }
    "#;

    let discovery = NginxDiscovery::from_config_text(config).unwrap();
    let servers = discovery.servers();

    assert_eq!(servers.len(), 2);
    assert!(servers[0].is_default_server());
    assert!(!servers[1].is_default_server());
}

#[test]
fn test_server_with_multiple_listen_directives() {
    let config = r#"
        server {
            listen 80;
            listen 443 ssl;
            listen 8080;
            server_name example.com;
        }
    "#;

    let discovery = NginxDiscovery::from_config_text(config).unwrap();
    let servers = discovery.servers();

    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].listen.len(), 3);
    assert!(servers[0].has_ssl());

    let ports = discovery.listening_ports();
    assert_eq!(ports.len(), 3); // 80, 443, 8080
}

#[test]
fn test_server_with_logs() {
    let config = r#"
        server {
            listen 80;
            server_name example.com;

            access_log /var/log/nginx/example.access.log;
            error_log /var/log/nginx/example.error.log;

            location /api {
                access_log /var/log/nginx/api.access.log;
            }
        }
    "#;

    let discovery = NginxDiscovery::from_config_text(config).unwrap();
    let servers = discovery.servers();

    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].access_logs.len(), 1);
    assert_eq!(servers[0].error_logs.len(), 1);

    // Location should also have its access log
    assert_eq!(servers[0].locations.len(), 1);
    assert_eq!(servers[0].locations[0].access_logs.len(), 1);
}

#[test]
fn test_location_modifiers() {
    let config = r#"
        server {
            location / {
                root /var/www;
            }
            location = /exact {
                return 200;
            }
            location ^~ /prefix {
                root /var/www/prefix;
            }
            location ~ \.php$ {
                fastcgi_pass unix:/var/run/php-fpm.sock;
            }
            location ~* \.jpg$ {
                expires 30d;
            }
        }
    "#;

    let discovery = NginxDiscovery::from_config_text(config).unwrap();
    let servers = discovery.servers();

    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].locations.len(), 5);

    // Check modifiers are parsed correctly
    use nginx_discovery::types::LocationModifier;
    assert_eq!(servers[0].locations[0].modifier, LocationModifier::None);
    assert_eq!(servers[0].locations[1].modifier, LocationModifier::Exact);
    assert_eq!(
        servers[0].locations[2].modifier,
        LocationModifier::PrefixPriority
    );
    assert_eq!(servers[0].locations[3].modifier, LocationModifier::Regex);
    assert_eq!(
        servers[0].locations[4].modifier,
        LocationModifier::RegexCaseInsensitive
    );
}
#[test]
fn test_empty_server_block() {
    let config = r#"
        server {
        }
    "#;

    let discovery = NginxDiscovery::from_config_text(config).unwrap();
    let servers = discovery.servers();

    assert_eq!(servers.len(), 1);
    assert!(servers[0].server_names.is_empty());
    assert!(servers[0].listen.is_empty());
    assert!(servers[0].locations.is_empty());
}
