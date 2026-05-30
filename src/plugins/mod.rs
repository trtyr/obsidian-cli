//! Plugin dispatch — routes CLI commands to plugin handlers.

use anyhow::Result;

use crate::cli::*;
use crate::kernel::vault::Vault;

pub mod batch;
pub mod bookmarks;
pub mod daily;
pub mod files;
pub mod links;
pub mod misc;
pub mod properties;
pub mod search;
pub mod tags;
pub mod tasks;
pub mod templates;

/// Execute a CLI command by dispatching to the appropriate plugin.
pub fn execute(cli: Cli, vault: Vault) -> Result<()> {
    match cli.command {
        Commands::Note { action } => files::handle(&vault, action),
        Commands::Link { action } => links::handle(&vault, action),
        Commands::Tag { action } => tags::handle(&vault, action),
        Commands::Prop { action } => properties::handle(&vault, action),
        Commands::Task { action } => tasks::handle(&vault, action),
        Commands::Daily { action } => daily::handle(&vault, action),
        Commands::Search {
            query,
            regex,
            case_sensitive,
            context,
            path_only,
            tag,
            max_results,
        } => search::handle(
            &vault,
            &query,
            regex,
            case_sensitive,
            context,
            path_only,
            tag,
            max_results,
        ),
        Commands::Template { action } => templates::handle(&vault, action),
        Commands::Bookmark { action } => bookmarks::handle(&vault, action),
        Commands::Batch { action } => batch::handle(&vault, action),
        Commands::Vault { action } => misc::handle_vault(&vault, action),
        Commands::Outline { note } => misc::handle_outline(&vault, &note),
        Commands::Wordcount { note } => misc::handle_wordcount(&vault, note),
        Commands::Recent { count } => misc::handle_recent(&vault, count),
    }
}
