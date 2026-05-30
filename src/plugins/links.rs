//! Link analysis plugin.

use anyhow::Result;
use colored::Colorize;

use crate::cli::LinkAction;
use crate::kernel::fs;
use crate::kernel::index::VaultIndex;
use crate::kernel::output;
use crate::kernel::vault::Vault;

/// Dispatch link actions.
pub fn handle(vault: &Vault, action: LinkAction) -> Result<()> {
    match action {
        LinkAction::Outgoing { note } => cmd_outgoing(vault, &note),
        LinkAction::Backlinks { note } => cmd_backlinks(vault, &note),
        LinkAction::Unresolved => cmd_unresolved(vault),
        LinkAction::Deadends => cmd_deadends(vault),
        LinkAction::Orphans => cmd_orphans(vault),
        LinkAction::Rename {
            old_name,
            new_name,
            dry_run,
            force,
        } => cmd_rename(vault, &old_name, &new_name, dry_run, force),
    }
}

fn cmd_outgoing(vault: &Vault, note_path: &str) -> Result<()> {
    let path = fs::resolve_note(vault, note_path)?;
    let note = crate::kernel::note::Note::parse(&path)?;
    println!("{} links from {}:", note.wikilinks.len(), note.name);
    println!("{}", output::format_links(&note.wikilinks));
    Ok(())
}

fn cmd_backlinks(vault: &Vault, note_path: &str) -> Result<()> {
    let path = fs::resolve_note(vault, note_path)?;
    let name = Vault::note_name(&path);
    let index = VaultIndex::build(vault)?;

    let backlinks = index.get_backlinks(&name);
    if backlinks.is_empty() {
        println!("No backlinks to {}", name);
    } else {
        println!("{} backlink(s) to {}:", backlinks.len(), name.bold());
        for note in &backlinks {
            println!("  {} {}", "←".cyan(), note.name);
        }
    }
    Ok(())
}

fn cmd_unresolved(vault: &Vault) -> Result<()> {
    let index = VaultIndex::build(vault)?;
    if index.unresolved.is_empty() {
        println!("{}", "No unresolved links.".green());
    } else {
        println!("{} unresolved link(s):", index.unresolved.len());
        for (source, target) in &index.unresolved {
            println!("  {} → {}", vault.relative_path(source), target.red());
        }
    }
    Ok(())
}

fn cmd_deadends(vault: &Vault) -> Result<()> {
    let index = VaultIndex::build(vault)?;
    let deadends = index.deadends();
    if deadends.is_empty() {
        println!("{}", "No dead-end notes.".green());
    } else {
        println!("{} dead-end note(s):", deadends.len());
        for note in &deadends {
            println!("  {}", note.name);
        }
    }
    Ok(())
}

fn cmd_orphans(vault: &Vault) -> Result<()> {
    let index = VaultIndex::build(vault)?;
    let orphans = index.orphans();
    if orphans.is_empty() {
        println!("{}", "No orphan notes.".green());
    } else {
        println!("{} orphan note(s):", orphans.len());
        for note in &orphans {
            println!("  {}", note.name);
        }
    }
    Ok(())
}

/// Rename a note and update all references to it.
fn cmd_rename(
    vault: &Vault,
    old_name: &str,
    new_name: &str,
    dry_run: bool,
    force: bool,
) -> Result<()> {
    let old_path = fs::resolve_note(vault, old_name)?;

    // Build new path
    let new_path = if new_name.contains('/') || new_name.contains('\\') {
        // Full path provided
        let p = vault.root.join(new_name);
        if p.extension().is_none() {
            p.with_extension("md")
        } else {
            p
        }
    } else {
        // Just a name, keep in same directory
        old_path.parent()
            .ok_or_else(|| anyhow::anyhow!("Could not determine parent directory"))?
            .join(if new_name.ends_with(".md") {
                new_name.to_string()
            } else {
                format!("{}.md", new_name)
            })
    };

    if new_path.exists() && !force {
        anyhow::bail!("Destination already exists: {}", new_path.display());
    }

    let old_note_name = Vault::note_name(&old_path);
    let new_note_name = Vault::note_name(&new_path);

    // Find all notes that reference the old name
    let index = VaultIndex::build(vault)?;
    let backlinks = index.get_backlinks(&old_note_name);

    if dry_run {
        println!("{} Would rename:", "Dry run:".yellow().bold());
        println!("  {} → {}", old_note_name, new_note_name);
        println!("  {} → {}", vault.relative_path(&old_path), vault.relative_path(&new_path));

        if !backlinks.is_empty() {
            println!("\nWould update {} reference(s):", backlinks.len());
            for note in &backlinks {
                println!("  {} {}", "←".cyan(), note.name);
            }
        }
        return Ok(());
    }

    // Confirm if not forced
    if !force {
        eprintln!("{} Rename {} → {}?", "⚠".yellow(), old_note_name, new_note_name);
        if !backlinks.is_empty() {
            eprintln!("  This will also update {} reference(s).", backlinks.len());
        }
        eprint!("  Continue? [y/N] ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    // Ensure parent directory exists
    if let Some(parent) = new_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Update references in all backlinking notes
    let mut updated_count = 0;
    for note in &backlinks {
        let content = std::fs::read_to_string(&note.path)?;
        let old_link = format!("[[{}]]", old_note_name);
        let new_link = format!("[[{}]]", new_note_name);

        if content.contains(&old_link) {
            let new_content = content.replace(&old_link, &new_link);
            std::fs::write(&note.path, &new_content)?;
            updated_count += 1;
            println!("  {} Updated: {}", "✓".green(), note.name);
        }

        // Also handle embedded links
        let old_embed = format!("![[{}]]", old_note_name);
        let new_embed = format!("![[{}]]", new_note_name);
        if content.contains(&old_embed) {
            let new_content = content.replace(&old_embed, &new_embed);
            std::fs::write(&note.path, &new_content)?;
            if !content.contains(&old_link) {
                updated_count += 1;
            }
            println!("  {} Updated embed: {}", "✓".green(), note.name);
        }
    }

    // Rename the file
    std::fs::rename(&old_path, &new_path)?;

    println!(
        "\n{} Renamed: {} → {}",
        "✓".green().bold(),
        vault.relative_path(&old_path),
        vault.relative_path(&new_path)
    );
    if updated_count > 0 {
        println!("  Updated {} reference(s).", updated_count);
    }

    Ok(())
}
