//! Command-line argument definitions

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// NGINX configuration discovery and analysis tool
#[derive(Parser, Debug)]
#[command(name = "nginx-discover")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalOpts,

    #[command(subcommand)]
    pub command: Commands,
}

/// Global options available to all commands
#[derive(Args, Debug)]
pub struct GlobalOpts {
    /// Path to nginx.conf (auto-detected if not specified)
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress all output except errors
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// When to use colored output
    #[arg(long, value_enum, default_value = "auto", global = true)]
    pub color: ColorChoice,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}

/// Available commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Parse and validate NGINX configuration
    Parse(ParseArgs),

    /// Extract specific information from configuration
    Extract(ExtractArgs),

    /// Analyze configuration for issues and best practices
    Analyze(AnalyzeArgs),

    /// Export configuration to different formats
    Export(ExportArgs),

    /// Run diagnostics and health checks
    Doctor(DoctorArgs),

    /// Interactive mode - guided configuration analysis
    Interactive,
}

/// Arguments for the parse command
#[derive(Args, Debug)]
pub struct ParseArgs {
    /// Display configuration as a tree
    #[arg(short, long)]
    pub tree: bool,

    /// Show summary only
    #[arg(short, long)]
    pub summary: bool,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

/// Arguments for the extract command
#[derive(Args, Debug)]
pub struct ExtractArgs {
    #[command(subcommand)]
    pub target: ExtractTarget,

    /// Output format
    #[arg(short, long, value_enum, default_value = "table", global = true)]
    pub format: OutputFormat,

    /// Output file (stdout if not specified)
    #[arg(short, long, global = true)]
    pub output: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum ExtractTarget {
    /// Extract server blocks
    Servers {
        /// Show only SSL-enabled servers
        #[arg(long)]
        ssl_only: bool,

        /// Filter by port
        #[arg(long)]
        port: Option<u16>,

        /// Filter by server name (supports wildcards)
        #[arg(long)]
        name: Option<String>,

        /// Output format
        #[arg(short, long, value_enum)]
        format: Option<OutputFormat>,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Extract log configurations
    Logs {
        /// Include log format definitions
        #[arg(long)]
        with_formats: bool,

        /// Filter by context (http, server, location)
        #[arg(long)]
        context: Option<String>,

        /// Output format
        #[arg(short, long, value_enum)]
        format: Option<OutputFormat>,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Extract location blocks
    Locations {
        /// Show only proxy locations
        #[arg(long)]
        proxy_only: bool,

        /// Show only static file locations
        #[arg(long)]
        static_only: bool,

        /// Filter by server name
        #[arg(long)]
        server: Option<String>,

        /// Output format
        #[arg(short, long, value_enum)]
        format: Option<OutputFormat>,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
    Csv,
}

/// Arguments for the export command
#[derive(Args, Debug)]
pub struct ExportArgs {
    /// Export format
    #[arg(value_enum)]
    pub format: ExportFormat,

    /// Output file (stdout if not specified)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Pretty-print output (for JSON/YAML)
    #[arg(long)]
    pub pretty: bool,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ExportFormat {
    Json,
    Yaml,
}

/// Arguments for the doctor command
#[derive(Args, Debug)]
pub struct DoctorArgs {
    /// Skip network checks
    #[arg(long)]
    pub no_network: bool,

    /// Attempt to fix issues automatically
    #[arg(long)]
    pub fix: bool,
}

/// Arguments for the analyze command
#[derive(Args, Debug)]
pub struct AnalyzeArgs {
    #[command(subcommand)]
    pub target: AnalyzeTarget,
}

#[derive(Subcommand, Debug)]
pub enum AnalyzeTarget {
    /// Analyze SSL/TLS configuration
    Ssl {
        /// Show only warnings and errors
        #[arg(long)]
        warnings_only: bool,

        /// Check if certificate files exist
        #[arg(long)]
        check_certs: bool,

        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        format: OutputFormat,

        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Analyze security configuration
    Security {
        /// Minimum severity level (info, warning, critical)
        #[arg(long, default_value = "info")]
        level: String,

        /// Show fix suggestions
        #[arg(long)]
        fix: bool,

        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        format: OutputFormat,

        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}
