//! Tag operations plugin — list, notes, add, remove, rename.

use anyhow::Result;
use colored::Colorize;

use crate::cli::TagAction;
use crate::kernel::fs;
use crate::kernel::index::VaultIndex;
use crate::kernel::note::Note;
use crate::kernel::vault::Vault;

/// Dispatch tag actions.
pub fn handle(vault: &Vault, action: TagAction) -> Result<()> {
    match action {
        TagAction::List { sort } => cmd_list(vault, sort),
        TagAction::Notes { tag } => cmd_notes(vault, &tag),
        TagAction::Add { note, tag } => cmd_add(vault, &note, &tag),
        TagAction::Remove { note, tag } => cmd_remove(vault, &note, &tag),
        TagAction::Rename { old, new, dry_run } => cmd_rename(vault, &old, &new, dry_run),
    }
}

fn cmd_list(vault: &Vault, sort: bool) -> Result<()> {
    let index = VaultIndex::build(vault)?;
    let mut tags = index.all_tags();

    if sort {
        tags.sort_by(|a, b| b.1.cmp(&a.1));
    }

    if tags.is_empty() {
        println!("{}", "No tags found.".dimmed());
    } else {
        println!("{} tag(s):", tags.len());
        for (tag, count) in &tags {
            println!("  #{} ({})", tag.cyan(), count);
        }
    }
    Ok(())
}

fn cmd_notes(vault: &Vault, tag: &str) -> Result<()> {
    let index = VaultIndex::build(vault)?;
    let notes = index.find_by_tag(tag);

    if notes.is_empty() {
        println!("No notes with tag #{}", tag);
    } else {
        println!("{} note(s) with #{}:", notes.len(), tag.cyan());
        for note in &notes {
            println!("  {}", note.name);
        }
    }
    Ok(())
}

fn cmd_add(vault: &Vault, note_path: &str, tag: &str) -> Result<()> {
    let path = fs::resolve_note(vault, note_path)?;
    let content = std::fs::read_to_string(&path)?;
    let note = Note::parse_str(&path, &content)?;

    // Check if tag already exists
    if note.has_tag(tag) {
        println!("Tag #{} already exists on {}", tag, note.name);
        return Ok(());
    }

    let mut existing_tags = note.property_tags();
    existing_tags.push(tag.to_string());

    let new_content = rebuild_frontmatter_with_tags(&content, &existing_tags)?;
    std::fs::write(&path, &new_content)?;
    println!("{} Added #{} to {}", "✓".green().bold(), tag.cyan(), note.name);
    Ok(())
}

fn cmd_remove(vault: &Vault, note_path: &str, tag: &str) -> Result<()> {
    let path = fs::resolve_note(vault, note_path)?;
    let content = std::fs::read_to_string(&path)?;
    let note = Note::parse_str(&path, &content)?;

    let existing_tags = note.property_tags();
    let new_tags: Vec<String> = existing_tags
        .iter()
        .filter(|t| *t != tag)
        .cloned()
        .collect();

    if new_tags.len() == existing_tags.len() {
        println!("Tag #{} not found on {}", tag, note.name);
        return Ok(());
    }

    let new_content = rebuild_frontmatter_with_tags(&content, &new_tags)?;
    std::fs::write(&path, &new_content)?;
    println!(
        "{} Removed #{} from {}",
        "✓".green().bold(),
        tag.cyan(),
        note.name
    );
    Ok(())
}

fn cmd_rename(vault: &Vault, old_tag: &str, new_tag: &str, dry_run: bool) -> Result<()> {
    let index = VaultIndex::build(vault)?;
    let notes = index.find_by_tag(old_tag);

    if notes.is_empty() {
        println!("No notes with tag #{}", old_tag);
        return Ok(());
    }

    println!(
        "{} note(s) with #{} → #{}:",
        notes.len(),
        old_tag.cyan(),
        new_tag.green()
    );

    for note in &notes {
        println!("  {}", note.name.dimmed());
    }

    if dry_run {
        println!("\n{} Dry run complete.", "✓".green());
        return Ok(());
    }

    let mut success = 0;
    for note in &notes {
        let content = std::fs::read_to_string(&note.path)?;
        let existing_tags = note.property_tags();
        let new_tags: Vec<String> = existing_tags
            .iter()
            .map(|t| if t == old_tag { new_tag.to_string() } else { t.clone() })
            .collect();

        let new_content = rebuild_frontmatter_with_tags(&content, &new_tags)?;
        std::fs::write(&note.path, &new_content)?;
        success += 1;
    }

    println!(
        "\n{} Renamed #{} → #{} in {} note(s).",
        "✓".green().bold(),
        old_tag.cyan(),
        new_tag.green(),
        success
    );
    Ok(())
}

/// Rebuild frontmatter with updated tags list.
fn rebuild_frontmatter_with_tags(content: &str, tags: &[String]) -> Result<String> {
    if content.starts_with("---") {
        if let Some(end) = content[3..].find("\n---") {
            let fm_end = 3 + end;
            let fm_content = &content[3..fm_end];
            let body = &content[fm_end..];

            let mut lines: Vec<String> = fm_content.lines().map(|l| l.to_string()).collect();

            // Remove existing tags section
            let mut in_tags = false;
            let mut tags_start = None;
            let mut tags_end = None;

            for (i, line) in lines.iter().enumerate() {
                if line.trim() == "tags:" {
                    in_tags = true;
                    tags_start = Some(i);
                    continue;
                }
                if in_tags {
                    if line.starts_with("  - ") || line.starts_with("- ") {
                        tags_end = Some(i);
                    } else {
                        in_tags = false;
                    }
                }
            }

            if let (Some(start), Some(end)) = (tags_start, tags_end) {
                lines.drain(start..=end);
            }

            // Add new tags
            if !tags.is_empty() {
                let mut tags_lines = vec!["tags:".to_string()];
                for tag in tags {
                    tags_lines.push(format!("  - {}", tag));
                }
                let insert_pos = if lines.is_empty() { 0 } else { 1 };
                for (i, line) in tags_lines.into_iter().enumerate() {
                    lines.insert(insert_pos + i, line);
                }
            }

            let new_fm = lines.join("\n");
            return Ok(format!("---\n{}{}", new_fm, body));
        }
    }

    // No frontmatter, create one
    let mut fm = "tags:\n".to_string();
    for tag in tags {
        fm.push_str(&format!("  - {}\n", tag));
    }
    Ok(format!("---\n{}---\n\n{}", fm, content))
}
