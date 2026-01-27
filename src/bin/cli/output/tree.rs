//! Tree view formatting for configuration

use colored::Colorize;
use nginx_discovery::ast::{Config, Directive};

pub fn print_tree(config: &Config) {
    println!("{}", "Configuration Tree:".bold());
    println!();

    for directive in &config.directives {
        print_directive(directive, 0, true);
    }
}

fn print_directive(directive: &Directive, level: usize, is_last: bool) {
    let indent = "  ".repeat(level);
    let prefix = if is_last { "└─" } else { "├─" };

    let name = directive.name().blue().bold();

    if directive.is_block() {
        // Block directive
        let args = directive.args_as_strings().join(" ");
        if args.is_empty() {
            println!("{}{} {} {{", indent, prefix, name);
        } else {
            println!("{}{} {} {} {{", indent, prefix, name, args.dimmed());
        }

        // Print children
        if let Some(children) = directive.children() {
            for (i, child) in children.iter().enumerate() {
                let is_last_child = i == children.len() - 1;
                print_directive(child, level + 1, is_last_child);
            }
        }

        println!("{}  }}", indent);
    } else {
        // Simple directive
        let args = directive.args_as_strings().join(" ");
        if args.is_empty() {
            println!("{}{} {};", indent, prefix, name);
        } else {
            println!("{}{} {} {};", indent, prefix, name, args.dimmed());
        }
    }
}
