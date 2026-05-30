//! obsidian-cli — A blazing-fast CLI for Obsidian vaults.
//!
//! Works WITHOUT the Obsidian desktop app — operates directly on vault files.
//!
//! Architecture: Microkernel
//! - `kernel/` — Core modules (vault, note, index, search, output, fs, config)
//! - `plugins/` — Feature modules (files, links, tags, tasks, etc.)

mod cli;
mod kernel;
mod plugins;

use clap::Parser;
use cli::{Cli, Commands, VaultAction};
use colored::Colorize;

fn main() {
    let cli = Cli::parse();

    // Special case: `vault set`, `vault unset`, `vault list` don't require a loaded vault
    if let Commands::Vault { ref action } = cli.command {
        match action {
            VaultAction::Set { path } => {
                if let Err(e) = plugins::misc::cmd_set_direct(path.as_deref()) {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
                return;
            }
            VaultAction::Unset => {
                if let Err(e) = plugins::misc::cmd_unset() {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
                return;
            }
            VaultAction::List => {
                if let Err(e) = plugins::misc::cmd_list() {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
                return;
            }
            _ => {} // Other vault commands need a loaded vault
        }
    }

    // Discover vault: --vault flag > config default > auto-discover
    let vault = match &cli.vault {
        // 1. Explicit --vault flag
        Some(path) => match kernel::vault::Vault::open(path) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
        },
        None => {
            // 2. Check config for default vault
            let config_vault = kernel::config::Config::load()
                .ok()
                .and_then(|c| c.get_vault().map(|s| s.to_string()));

            match config_vault {
                Some(path) => match kernel::vault::Vault::open(&path) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("{} {}", "Error:".red().bold(), e);
                        eprintln!(
                            "{} Configured vault not found: {}",
                            "Hint:".yellow(),
                            path
                        );
                        eprintln!("       Run `vault set` to update the default vault path.");
                        std::process::exit(1);
                    }
                },
                None => {
                    // 3. Auto-discover from current directory
                    match kernel::vault::Vault::discover() {
                        Ok(v) => v,
                        Err(e) => {
                            eprintln!("{} {}", "Error:".red().bold(), e);
                            eprintln!(
                                "{} Run this command inside an Obsidian vault, use --vault <path>, or run `vault set` to configure a default.",
                                "Hint:".yellow()
                            );
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
    };

    // Execute command
    if let Err(e) = plugins::execute(cli, vault) {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}
