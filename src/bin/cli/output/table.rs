//! Table formatting for CLI output

use nginx_discovery::types::{AccessLog, Location, LogFormat, Server};
use tabled::{settings::Style, Table, Tabled};

#[derive(Tabled)]
struct ServerRow {
    #[tabled(rename = "Server Name")]
    name: String,
    #[tabled(rename = "Port")]
    port: String,
    #[tabled(rename = "SSL")]
    ssl: String,
    #[tabled(rename = "Locations")]
    locations: usize,
    #[tabled(rename = "Default")]
    default: String,
}

pub fn format_servers(servers: &[Server]) -> String {
    if servers.is_empty() {
        return "No servers found.".to_string();
    }

    let mut rows = Vec::new();

    for server in servers {
        let name = server.primary_name().unwrap_or("_").to_string();

        for listen in &server.listen {
            rows.push(ServerRow {
                name: name.clone(),
                port: listen.port.to_string(),
                ssl: if listen.ssl { "Yes" } else { "No" }.to_string(),
                locations: server.locations.len(),
                default: if listen.default_server { "Yes" } else { "No" }.to_string(),
            });
        }

        // If no listen directives, still show the server
        if server.listen.is_empty() {
            rows.push(ServerRow {
                name: name.clone(),
                port: "-".to_string(),
                ssl: "No".to_string(),
                locations: server.locations.len(),
                default: "No".to_string(),
            });
        }
    }

    let mut table = Table::new(rows);
    table.with(Style::rounded());

    let table_str = table.to_string();
    let ssl_count = servers.iter().filter(|s| s.has_ssl()).count();

    format!(
        "{}\n\nTotal: {} servers ({} with SSL)",
        table_str,
        servers.len(),
        ssl_count
    )
}

pub fn format_servers_csv(servers: &[Server]) -> String {
    let mut output = String::from("Server Name,Port,SSL,Locations,Default\n");

    for server in servers {
        let name = server.primary_name().unwrap_or("_");

        for listen in &server.listen {
            output.push_str(&format!(
                "{},{},{},{},{}\n",
                name,
                listen.port,
                if listen.ssl { "Yes" } else { "No" },
                server.locations.len(),
                if listen.default_server { "Yes" } else { "No" }
            ));
        }
    }

    output
}

#[derive(Tabled)]
struct LogRow {
    #[tabled(rename = "Log File")]
    path: String,
    #[tabled(rename = "Format")]
    format: String,
    #[tabled(rename = "Context")]
    context: String,
}

pub fn format_logs(logs: &[AccessLog], formats: Option<&[LogFormat]>) -> String {
    if logs.is_empty() {
        return "No log files found.".to_string();
    }

    let rows: Vec<LogRow> = logs
        .iter()
        .map(|log| LogRow {
            path: log.path.display().to_string(),
            format: log
                .format_name
                .clone()
                .unwrap_or_else(|| "combined".to_string()),
            context: format!("{:?}", log.context),
        })
        .collect();

    let mut table = Table::new(rows);
    table.with(Style::rounded());

    let mut output = table.to_string();
    output.push_str(&format!("\n\nTotal: {} log files", logs.len()));

    if let Some(fmts) = formats {
        output.push_str("\n\nLog Formats:\n");
        for fmt in fmts {
            output.push_str(&format!("\n  {}:\n", fmt.name()));
            output.push_str(&format!("    Variables: {}\n", fmt.variables().join(", ")));
        }
    }

    output
}

pub fn format_logs_csv(logs: &[AccessLog]) -> String {
    let mut output = String::from("Log File,Format,Context\n");

    for log in logs {
        output.push_str(&format!(
            "{},{},{:?}\n",
            log.path.display(),
            log.format_name.as_deref().unwrap_or("combined"),
            log.context
        ));
    }

    output
}

#[derive(Tabled)]
struct LocationRow {
    #[tabled(rename = "Server")]
    server: String,
    #[tabled(rename = "Path")]
    path: String,
    #[tabled(rename = "Modifier")]
    modifier: String,
    #[tabled(rename = "Type")]
    location_type: String,
    #[tabled(rename = "Target")]
    target: String,
}

pub fn format_locations(locations: &[(String, Location)]) -> String {
    if locations.is_empty() {
        return "No locations found.".to_string();
    }

    let rows: Vec<LocationRow> = locations
        .iter()
        .map(|(server, loc)| {
            let (loc_type, target) = if loc.is_proxy() {
                (
                    "Proxy".to_string(),
                    loc.proxy_pass.clone().unwrap_or_else(|| "-".to_string()),
                )
            } else if loc.is_static() {
                let root_str = loc
                    .root
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "-".to_string());
                ("Static".to_string(), root_str)
            } else {
                ("Other".to_string(), "-".to_string())
            };

            LocationRow {
                server: server.clone(),
                path: loc.path.clone(),
                modifier: format!("{:?}", loc.modifier),
                location_type: loc_type,
                target,
            }
        })
        .collect();

    let mut table = Table::new(rows);
    table.with(Style::rounded());

    format!("{}\n\nTotal: {} locations", table, locations.len())
}

pub fn format_locations_csv(locations: &[(String, Location)]) -> String {
    let mut output = String::from("Server,Path,Modifier,Type,Target\n");

    for (server, loc) in locations {
        let (loc_type, target) = if loc.is_proxy() {
            (
                "Proxy".to_string(),
                loc.proxy_pass.clone().unwrap_or_else(|| "-".to_string()),
            )
        } else if loc.is_static() {
            let root_str = loc
                .root
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "-".to_string());
            ("Static".to_string(), root_str)
        } else {
            ("Other".to_string(), "-".to_string())
        };

        output.push_str(&format!(
            "{},{},{:?},{},{}\n",
            server, loc.path, loc.modifier, loc_type, target
        ));
    }

    output
}
