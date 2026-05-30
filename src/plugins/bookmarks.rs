//! Bookmark plugin.

use anyhow::Result;
use colored::Colorize;

use crate::cli::BookmarkAction;
use crate::kernel::vault::Vault;

/// Dispatch bookmark actions.
pub fn handle(vault: &Vault, action: BookmarkAction) -> Result<()> {
    match action {
        BookmarkAction::List => cmd_list(vault),
    }
}

fn cmd_list(vault: &Vault) -> Result<()> {
    let bookmarks_path = vault.root.join(".obsidian").join("bookmarks.json");
    if !bookmarks_path.exists() {
        println!("{}", "No bookmarks found.".dimmed());
        return Ok(());
    }

    let content = std::fs::read_to_string(&bookmarks_path)?;
    let bookmarks: serde_json::Value = serde_json::from_str(&content)?;

    println!("{}", "Bookmarks:".bold());
    print_bookmark_items(&bookmarks, 0);
    Ok(())
}

fn print_bookmark_items(value: &serde_json::Value, indent: usize) {
    let prefix = "  ".repeat(indent);
    match value {
        serde_json::Value::Array(arr) => {
            for item in arr {
                print_bookmark_items(item, indent);
            }
        }
        serde_json::Value::Object(map) => {
            if let Some(items) = map.get("items") {
                let name = map
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Untitled");
                println!("{}📁 {}", prefix, name.bold());
                print_bookmark_items(items, indent + 1);
            } else {
                let name = map
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Untitled");
                let path = map.get("path").and_then(|v| v.as_str()).unwrap_or("");
                println!("{}  {} {}", prefix, "→".cyan(), name);
                if !path.is_empty() {
                    println!("{}    {}", prefix, path.dimmed());
                }
            }
        }
        _ => {}
    }
}
