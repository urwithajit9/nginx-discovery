// tests/export_integration.rs
//! Integration tests for export functionality

use nginx_discovery::{parse, export::{export, ExportOptions, ExportFormat, Filter, FilterType}};
use std::io::Cursor;

const SAMPLE_CONFIG: &str = r#"
http {
    server {
        listen 80;
        listen 443 ssl;
        server_name example.com www.example.com;
        root /var/www/html;

        ssl_certificate /etc/ssl/cert.pem;
        ssl_certificate_key /etc/ssl/key.pem;

        location / {
            proxy_pass http://backend;
        }
    }

    server {
        listen 8080;
        server_name api.example.com;

        location /api {
            proxy_pass http://api_backend;
        }
    }
}
"#;

#[test]
fn test_export_json() {
    let config = parse(SAMPLE_CONFIG).expect("Failed to parse config");
    let options = ExportOptions::builder()
        .format(ExportFormat::Json)
        .pretty(true)
        .build();

    let mut output = Cursor::new(Vec::new());
    export(&config, &mut output, options).expect("Failed to export");

    let json_str = String::from_utf8(output.into_inner()).unwrap();
    assert!(!json_str.is_empty());
    assert!(json_str.contains("directives"));
}

#[test]
fn test_export_yaml() {
    let config = parse(SAMPLE_CONFIG).expect("Failed to parse config");
    let options = ExportOptions::builder()
        .format(ExportFormat::Yaml)
        .build();

    let mut output = Cursor::new(Vec::new());
    export(&config, &mut output, options).expect("Failed to export");

    let yaml_str = String::from_utf8(output.into_inner()).unwrap();
    assert!(!yaml_str.is_empty());
}

#[test]
#[cfg(feature = "export-toml")]
fn test_export_toml() {
    let config = parse(SAMPLE_CONFIG).expect("Failed to parse config");
    let options = ExportOptions::builder()
        .format(ExportFormat::Toml)
        .pretty(true)
        .build();

    let mut output = Cursor::new(Vec::new());
    export(&config, &mut output, options).expect("Failed to export");

    let toml_str = String::from_utf8(output.into_inner()).unwrap();
    assert!(!toml_str.is_empty());
}

#[test]
#[cfg(feature = "export-markdown")]
fn test_export_markdown() {
    let config = parse(SAMPLE_CONFIG).expect("Failed to parse config");
    let options = ExportOptions::builder()
        .format(ExportFormat::Markdown)
        .include_metadata(true)
        .build();

    let mut output = Cursor::new(Vec::new());
    export(&config, &mut output, options).expect("Failed to export");

    let md_str = String::from_utf8(output.into_inner()).unwrap();
    assert!(!md_str.is_empty());
    assert!(md_str.contains("# NGINX Configuration Report"));
    assert!(md_str.contains("## HTTP Configuration"));
}

#[test]
fn test_export_with_filter() {
    let config = parse(SAMPLE_CONFIG).expect("Failed to parse config");

    let filter = Filter::new(FilterType::ServerName, "example.com");
    let options = ExportOptions::builder()
        .format(ExportFormat::Json)
        .filter(filter)
        .build();

    let mut output = Cursor::new(Vec::new());
    export(&config, &mut output, options).expect("Failed to export");

    let json_str = String::from_utf8(output.into_inner()).unwrap();
    assert!(!json_str.is_empty());
}

#[test]
fn test_parse_filter_from_string() {
    let filter: Filter = "server_name=*.example.com".parse().unwrap();
    assert_eq!(filter.filter_type, FilterType::ServerName);
    assert_eq!(filter.pattern, "*.example.com");
}

#[test]
fn test_export_options_builder() {
    let options = ExportOptions::builder()
        .format(ExportFormat::Json)
        .pretty(true)
        .compact(false)
        .include_metadata(true)
        .include_comments(false)
        .build();

    assert_eq!(options.format, ExportFormat::Json);
    assert!(options.pretty);
    assert!(!options.compact);
    assert!(options.include_metadata);
    assert!(!options.include_comments);
}

#[test]
fn test_format_from_str() {
    assert_eq!("json".parse::<ExportFormat>().unwrap(), ExportFormat::Json);
    assert_eq!("yaml".parse::<ExportFormat>().unwrap(), ExportFormat::Yaml);
    assert_eq!("yml".parse::<ExportFormat>().unwrap(), ExportFormat::Yaml);

    #[cfg(feature = "export-toml")]
    {
        assert_eq!("toml".parse::<ExportFormat>().unwrap(), ExportFormat::Toml);
    }

    #[cfg(feature = "export-markdown")]
    {
        assert_eq!("markdown".parse::<ExportFormat>().unwrap(), ExportFormat::Markdown);
        assert_eq!("md".parse::<ExportFormat>().unwrap(), ExportFormat::Markdown);
    }
}