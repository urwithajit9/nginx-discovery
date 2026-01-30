// src/bin/cli/commands/completions.rs
//! Shell completion generation command

use crate::cli::args::Cli;
use clap::{CommandFactory, ValueEnum};
use clap_complete::{generate, Shell};
use std::io;

/// Shell types for completion generation
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CompletionShell {
    /// Bash shell
    Bash,
    /// Zsh shell
    Zsh,
    /// Fish shell
    Fish,
    /// PowerShell
    PowerShell,
    /// Elvish shell
    Elvish,
}

impl From<CompletionShell> for Shell {
    fn from(shell: CompletionShell) -> Self {
        match shell {
            CompletionShell::Bash => Shell::Bash,
            CompletionShell::Zsh => Shell::Zsh,
            CompletionShell::Fish => Shell::Fish,
            CompletionShell::PowerShell => Shell::PowerShell,
            CompletionShell::Elvish => Shell::Elvish,
        }
    }
}

/// Arguments for completions command
#[derive(Debug, clap::Args)]
pub struct CompletionsArgs {
    /// Shell to generate completions for
    #[arg(value_enum)]
    pub shell: CompletionShell,
    
    /// Output file (stdout if not specified)
    #[arg(short, long)]
    pub output: Option<std::path::PathBuf>,
}

/// Run completions generation command
pub fn run(args: CompletionsArgs) -> anyhow::Result<()> {
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    
    if let Some(output_path) = args.output {
        // Write to file
        let mut file = std::fs::File::create(&output_path)?;
        generate(args.shell.into(), &mut cmd, &bin_name, &mut file);
        
        println!("Generated completions for {:?} to {}", 
            args.shell, output_path.display());
        
        print_installation_instructions(args.shell, &output_path);
    } else {
        // Write to stdout
        generate(args.shell.into(), &mut cmd, &bin_name, &mut io::stdout());
    }
    
    Ok(())
}

/// Print installation instructions for the generated completions
fn print_installation_instructions(shell: CompletionShell, path: &std::path::Path) {
    println!();
    println!("Installation instructions:");
    println!();
    
    match shell {
        CompletionShell::Bash => {
            println!("  For system-wide installation:");
            println!("    sudo cp {} /etc/bash_completion.d/", path.display());
            println!();
            println!("  For user installation:");
            println!("    mkdir -p ~/.local/share/bash-completion/completions");
            println!("    cp {} ~/.local/share/bash-completion/completions/nginx-discover", path.display());
            println!();
            println!("  Or source it in your ~/.bashrc:");
            println!("    echo 'source {}' >> ~/.bashrc", path.display());
        }
        CompletionShell::Zsh => {
            println!("  For system-wide installation:");
            println!("    sudo cp {} /usr/share/zsh/site-functions/_nginx-discover", path.display());
            println!();
            println!("  For user installation:");
            println!("    mkdir -p ~/.zsh/completions");
            println!("    cp {} ~/.zsh/completions/_nginx-discover", path.display());
            println!("    echo 'fpath=(~/.zsh/completions $fpath)' >> ~/.zshrc");
            println!("    echo 'autoload -Uz compinit && compinit' >> ~/.zshrc");
        }
        CompletionShell::Fish => {
            println!("  For system-wide installation:");
            println!("    sudo cp {} /usr/share/fish/vendor_completions.d/nginx-discover.fish", path.display());
            println!();
            println!("  For user installation:");
            println!("    mkdir -p ~/.config/fish/completions");
            println!("    cp {} ~/.config/fish/completions/nginx-discover.fish", path.display());
        }
        CompletionShell::PowerShell => {
            println!("  Add to your PowerShell profile:");
            println!("    . {}", path.display());
            println!();
            println!("  To find your profile location:");
            println!("    echo $PROFILE");
        }
        CompletionShell::Elvish => {
            println!("  Add to your Elvish rc.elv:");
            println!("    eval (cat {})", path.display());
        }
    }
    
    println!();
    println!("After installation, restart your shell or source your shell configuration file.");
}

/// Quick setup function that automatically installs completions
pub fn quick_setup() -> anyhow::Result<()> {
    println!("nginx-discover Shell Completion Setup");
    println!("======================================");
    println!();
    
    // Detect shell
    let shell = detect_shell()?;
    println!("Detected shell: {:?}", shell);
    println!();
    
    // Generate and install
    let output_path = get_completion_path(shell)?;
    
    println!("Generating completions to: {}", output_path.display());
    
    let args = CompletionsArgs {
        shell,
        output: Some(output_path.clone()),
    };
    
    run(args)?;
    
    Ok(())
}

/// Detect the current shell
fn detect_shell() -> anyhow::Result<CompletionShell> {
    if let Ok(shell) = std::env::var("SHELL") {
        if shell.contains("bash") {
            return Ok(CompletionShell::Bash);
        } else if shell.contains("zsh") {
            return Ok(CompletionShell::Zsh);
        } else if shell.contains("fish") {
            return Ok(CompletionShell::Fish);
        }
    }
    
    // Default to bash
    Ok(CompletionShell::Bash)
}

/// Get appropriate completion file path for shell
fn get_completion_path(shell: CompletionShell) -> anyhow::Result<std::path::PathBuf> {
    let home = std::env::var("HOME")?;
    let path = match shell {
        CompletionShell::Bash => {
            std::path::PathBuf::from(home)
                .join(".local/share/bash-completion/completions/nginx-discover")
        }
        CompletionShell::Zsh => {
            std::path::PathBuf::from(home)
                .join(".zsh/completions/_nginx-discover")
        }
        CompletionShell::Fish => {
            std::path::PathBuf::from(home)
                .join(".config/fish/completions/nginx-discover.fish")
        }
        CompletionShell::PowerShell => {
            std::path::PathBuf::from(home)
                .join("Documents/PowerShell/Scripts/nginx-discover-completion.ps1")
        }
        CompletionShell::Elvish => {
            std::path::PathBuf::from(home)
                .join(".elvish/lib/nginx-discover.elv")
        }
    };
    
    // Create parent directory if it doesn't exist
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_completion_shell_conversion() {
        let bash: Shell = CompletionShell::Bash.into();
        assert_eq!(format!("{:?}", bash), "Bash");
    }
    
    #[test]
    fn test_detect_shell() {
        // This test depends on environment, so just ensure it doesn't panic
        let _ = detect_shell();
    }
}