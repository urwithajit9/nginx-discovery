//! nginx-discover CLI tool
//!
//! Command-line interface for NGINX configuration discovery

use anyhow::Result;
use clap::Parser;

mod cli;

use cli::{Cli, Commands};

fn main() -> Result<()> {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Execute the appropriate command
    match cli.command {
        Commands::Parse(args) => cli::commands::parse::run(args, &cli.global)?,
        Commands::Extract(args) => cli::commands::extract::run(args, &cli.global)?,
        Commands::Analyze(args) => cli::commands::analyze::run(args, &cli.global)?,
        Commands::Export(args) => cli::commands::export::run(args, &cli.global)?,
        Commands::Doctor(args) => cli::commands::doctor::run(args, &cli.global)?,
        Commands::Interactive => cli::commands::interactive::run(&cli.global)?,
    }

    Ok(())
}
