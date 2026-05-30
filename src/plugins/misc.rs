//! Misc plugin — vault info/stats/repair/export/set/unset/list, outline, wordcount, recent.

use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use colored::Colorize;
use walkdir::WalkDir;

use crate::cli::VaultAction;
use crate::kernel::config::Config;
use crate::kernel::fs;
use crate::kernel::index::VaultIndex;
use crate::kernel::note::Note;
use crate::kernel::output;
use crate::kernel::vault::Vault;

/// Dispatch vault-level actions.
pub fn handle_vault(vault: &Vault, action: VaultAction) -> Result<()> {
    match action {
        VaultAction::Info => cmd_info(vault),
        VaultAction::Stats => cmd_vault_stats(vault),
        VaultAction::Repair { fix_unresolved, remove_deadends, dry_run } => {
            cmd_repair(vault, fix_unresolved, remove_deadends, dry_run)
        }
        VaultAction::Export { output, include_content, pretty } => {
            cmd_export(vault, output, include_content, pretty)
        }
        VaultAction::Set { path } => cmd_set_from_vault(vault, path),
        VaultAction::Unset => cmd_unset(),
        VaultAction::List => cmd_list(),
    }
}

/// Show note outline.
pub fn handle_outline(vault: &Vault, note_path: &str) -> Result<()> {
    let path = fs::resolve_note(vault, note_path)?;
    let note = Note::parse(&path)?;
    println!("{}", output::format_outline(&note.headings));
    Ok(())
}

/// Show word count.
pub fn handle_wordcount(vault: &Vault, note: Option<String>) -> Result<()> {
    match note {
        Some(n) => {
            let path = fs::resolve_note(vault, &n)?;
            let note = Note::parse(&path)?;
            println!("{}", output::format_wordcount(&note));
        }
        None => {
            let index = VaultIndex::build(vault)?;
            let mut notes: Vec<_> = index.notes.values().collect();
            notes.sort_by(|a, b| b.word_count.cmp(&a.word_count));

            println!("{}", "Word Count by Note:".bold());
            for note in notes.iter().take(20) {
                println!("  {} - {} words", note.name, note.word_count);
            }

            let total: usize = index.notes.values().map(|n| n.word_count).sum();
            println!(
                "\n{} total words across {} notes",
                total,
                index.notes.len()
            );
        }
    }
    Ok(())
}

/// Show recent notes.
pub fn handle_recent(vault: &Vault, count: usize) -> Result<()> {
    let mut notes: Vec<_> = Vec::new();

    for entry in WalkDir::new(&vault.root)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !fs::is_hidden_entry(e))
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        if path.extension().map_or(true, |e| e != "md") {
            continue;
        }

        if path.starts_with(vault.root.join(".obsidian")) {
            continue;
        }

        if let Ok(meta) = std::fs::metadata(path) {
            if let Ok(modified) = meta.modified() {
                notes.push((path.to_path_buf(), modified));
            }
        }
    }

    notes.sort_by(|a, b| b.1.cmp(&a.1));
    notes.truncate(count);

    println!("{} recent notes:", count);
    for (path, time) in &notes {
        let datetime: DateTime<Local> = (*time).into();
        println!(
            "  {} {}",
            datetime.format("%Y-%m-%d %H:%M").to_string().dimmed(),
            vault.relative_path(path)
        );
    }

    Ok(())
}

fn cmd_info(vault: &Vault) -> Result<()> {
    println!("{}", "Vault Information:".bold());
    println!("  {} {}", "Path:".dimmed(), vault.root.display());
    println!("  {} {}", "Daily Notes:".dimmed(), vault.daily.folder);
    println!(
        "  {} {}",
        "New Files:".dimmed(),
        vault
            .app
            .new_file_folder_path
            .as_deref()
            .unwrap_or("(root)")
    );
    println!(
        "  {} {}",
        "Attachments:".dimmed(),
        vault
            .app
            .attachment_folder_path
            .as_deref()
            .unwrap_or("(root)")
    );
    println!(
        "  {} {}",
        "Update Links:".dimmed(),
        vault.app.always_update_links
    );
    Ok(())
}

fn cmd_vault_stats(vault: &Vault) -> Result<()> {
    let index = VaultIndex::build(vault)?;
    let total_notes = index.notes.len();
    let total_words: usize = index.notes.values().map(|n| n.word_count).sum();
    let total_links: usize = index.notes.values().map(|n| n.wikilinks.len()).sum();
    let total_tags = index.tag_index.len();
    let total_tasks: usize = index.notes.values().map(|n| n.tasks.len()).sum();
    let unresolved = index.unresolved.len();

    println!("{}", "Vault Statistics:".bold());
    println!("  {} {}", "Notes:".dimmed(), total_notes);
    println!("  {} {}", "Words:".dimmed(), total_words);
    println!("  {} {}", "Links:".dimmed(), total_links);
    println!("  {} {}", "Tags:".dimmed(), total_tags);
    println!("  {} {}", "Tasks:".dimmed(), total_tasks);
    println!("  {} {}", "Unresolved:".dimmed(), unresolved);

    Ok(())
}

/// Repair vault issues.
fn cmd_repair(vault: &Vault, fix_unresolved: bool, remove_deadends: bool, dry_run: bool) -> Result<()> {
    let index = VaultIndex::build(vault)?;

    if fix_unresolved {
        let unresolved = &index.unresolved;
        if unresolved.is_empty() {
            println!("{}", "No unresolved links to fix.".green());
        } else {
            println!("{} unresolved link(s) found:", unresolved.len());
            for (source, target) in unresolved {
                println!("  {} → {}", vault.relative_path(source), target.cyan());

                if !dry_run {
                    // Create a stub note for the unresolved target
                    let stub_path = vault.root.join(format!("{}.md", target));
                    if !stub_path.exists() {
                        if let Some(parent) = stub_path.parent() {
                            std::fs::create_dir_all(parent)?;
                        }
                        std::fs::write(&stub_path, format!("# {}\n\n> Auto-created by vault repair.\n", target))?;
                        println!("    {} Created stub: {}", "✓".green(), stub_path.display());
                    }
                }
            }

            if dry_run {
                println!("\n{} Run without --dry-run to apply fixes.", "Tip:".yellow());
            } else {
                println!("\n{} Fixed {} unresolved link(s).", "✓".green().bold(), unresolved.len());
            }
        }
    }

    if remove_deadends {
        let deadends = index.deadends();
        if deadends.is_empty() {
            println!("{}", "No dead-end notes to remove.".green());
        } else {
            println!("{} dead-end note(s) found:", deadends.len());
            for note in &deadends {
                println!("  {}", note.name);

                if !dry_run {
                    // Move to trash
                    #[cfg(target_os = "macos")]
                    {
                        let trash_dir = dirs::home_dir()
                            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
                            .join(".Trash");
                        let filename = note.path.file_name()
                            .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;
                        let dest = trash_dir.join(filename);
                        std::fs::rename(&note.path, &dest)?;
                        println!("    {} Moved to trash: {}", "✓".green(), note.name);
                    }
                    #[cfg(not(target_os = "macos"))]
                    {
                        std::fs::remove_file(&note.path)?;
                        println!("    {} Deleted: {}", "✓".green(), note.name);
                    }
                }
            }

            if dry_run {
                println!("\n{} Run without --dry-run to apply fixes.", "Tip:".yellow());
            } else {
                println!("\n{} Removed {} dead-end note(s).", "✓".green().bold(), deadends.len());
            }
        }
    }

    if !fix_unresolved && !remove_deadends {
        println!("{}", "No repair actions specified.".yellow());
        println!("Use --fix-unresolved to create stub notes for broken links.");
        println!("Use --remove-deadends to delete notes with no incoming links.");
    }

    Ok(())
}

/// Export vault data to JSON.
fn cmd_export(vault: &Vault, output: Option<String>, include_content: bool, pretty: bool) -> Result<()> {
    let index = VaultIndex::build(vault)?;

    let mut notes = Vec::new();
    for note in index.notes.values() {
        let mut note_data = serde_json::json!({
            "name": note.name,
            "path": vault.relative_path(&note.path),
            "word_count": note.word_count,
            "char_count": note.char_count,
            "tags": note.tags,
            "wikilinks": note.wikilinks.iter().map(|l| &l.target).collect::<Vec<_>>(),
            "tasks": note.tasks.iter().map(|t| serde_json::json!({
                "text": t.text,
                "done": t.done,
            })).collect::<Vec<_>>(),
            "headings": note.headings.iter().map(|h| serde_json::json!({
                "level": h.level,
                "text": h.text,
            })).collect::<Vec<_>>(),
        });

        if include_content {
            note_data["content"] = serde_json::Value::String(note.raw.clone());
        }

        notes.push(note_data);
    }

    let export = serde_json::json!({
        "vault": vault.root.display().to_string(),
        "exported_at": chrono::Local::now().to_rfc3339(),
        "note_count": notes.len(),
        "notes": notes,
    });

    let json = if pretty {
        serde_json::to_string_pretty(&export)?
    } else {
        serde_json::to_string(&export)?
    };

    match output {
        Some(path) => {
            std::fs::write(&path, &json)?;
            println!("{} Exported to {}", "✓".green().bold(), path);
        }
        None => {
            println!("{}", json);
        }
    }

    Ok(())
}

/// Set default vault path (with loaded vault).
fn cmd_set_from_vault(vault: &Vault, path: Option<String>) -> Result<()> {
    let vault_path = match path {
        Some(p) => {
            let p = std::path::PathBuf::from(&p);
            if !p.exists() {
                anyhow::bail!("Path does not exist: {}", p.display());
            }
            if !p.join(".obsidian").exists() {
                anyhow::bail!("Not an Obsidian vault: {} (no .obsidian directory)", p.display());
            }
            p.canonicalize()
                .context("Could not resolve path")?
                .display()
                .to_string()
        }
        None => {
            vault.root.canonicalize()
                .context("Could not resolve path")?
                .display()
                .to_string()
        }
    };

    let mut config = Config::load()?;
    config.set_vault(vault_path.clone());
    config.save()?;

    println!("{} Default vault set to: {}", "✓".green().bold(), vault_path);
    Ok(())
}

/// Set default vault path (direct call without loaded vault).
pub fn cmd_set_direct(path: Option<&str>) -> Result<()> {
    let vault_path = match path {
        Some(p) => {
            let p = std::path::PathBuf::from(p);
            if !p.exists() {
                anyhow::bail!("Path does not exist: {}", p.display());
            }
            if !p.join(".obsidian").exists() {
                anyhow::bail!("Not an Obsidian vault: {} (no .obsidian directory)", p.display());
            }
            p.canonicalize()
                .context("Could not resolve path")?
                .display()
                .to_string()
        }
        None => {
            let cwd = std::env::current_dir()
                .context("Could not determine current directory")?;
            if !cwd.join(".obsidian").exists() {
                anyhow::bail!("Not an Obsidian vault: {} (no .obsidian directory)", cwd.display());
            }
            cwd.canonicalize()
                .context("Could not resolve path")?
                .display()
                .to_string()
        }
    };

    let mut config = Config::load()?;
    config.set_vault(vault_path.clone());
    config.save()?;

    println!("{} Default vault set to: {}", "✓".green().bold(), vault_path);
    Ok(())
}

/// Remove default vault path from config.
pub fn cmd_unset() -> Result<()> {
    let mut config = Config::load()?;
    if config.get_vault().is_none() {
        println!("{}", "No default vault configured.".yellow());
        return Ok(());
    }

    config.unset_vault();
    config.save()?;

    println!("{} Default vault removed.", "✓".green().bold());
    Ok(())
}

/// List configured vaults.
pub fn cmd_list() -> Result<()> {
    let config = Config::load()?;

    match config.get_vault() {
        Some(path) => {
            println!("{}", "Default vault:".bold());
            println!("  {}", path);
        }
        None => {
            println!("{}", "No default vault configured.".dimmed());
        }
    }

    if !config.vaults.is_empty() {
        println!("\n{}", "Named vaults:".bold());
        let mut vaults: Vec<_> = config.vaults.iter().collect();
        vaults.sort_by_key(|(name, _)| (*name).clone());
        for (name, path) in vaults {
            println!("  {} → {}", name.cyan(), path);
        }
    }

    Ok(())
}
