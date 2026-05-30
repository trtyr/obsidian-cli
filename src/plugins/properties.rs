//! Frontmatter property operations plugin.

use anyhow::Result;
use colored::Colorize;
use regex;

use crate::cli::PropAction;
use crate::kernel::fs;
use crate::kernel::note::Note;
use crate::kernel::vault::Vault;

/// Dispatch property actions.
pub fn handle(vault: &Vault, action: PropAction) -> Result<()> {
    match action {
        PropAction::Get { note, key } => cmd_get(vault, &note, key),
        PropAction::Set { note, key, value } => cmd_set(vault, &note, &key, &value),
        PropAction::Remove { note, key } => cmd_remove(vault, &note, &key),
    }
}

fn cmd_get(vault: &Vault, note_path: &str, key: Option<String>) -> Result<()> {
    let path = fs::resolve_note(vault, note_path)?;
    let note = Note::parse(&path)?;

    match key {
        Some(k) => match note.property(&k) {
            Some(v) => println!("{}", v),
            None => println!("Property '{}' not found.", k),
        },
        None => {
            if note.frontmatter.is_empty() {
                println!("No properties.");
            } else {
                for (k, v) in &note.frontmatter {
                    println!("{}: {}", k.cyan(), v.to_display_string());
                }
            }
        }
    }
    Ok(())
}

fn cmd_set(vault: &Vault, note_path: &str, key: &str, value: &str) -> Result<()> {
    let path = fs::resolve_note(vault, note_path)?;
    let mut content = std::fs::read_to_string(&path)?;

    if content.starts_with("---") {
        if let Some(end) = content[3..].find("\n---") {
            let fm_end = 3 + end;
            let fm_content = &content[3..fm_end];

            let key_pattern = format!("{}:", key);
            if fm_content.contains(&key_pattern) {
                let re = regex::Regex::new(&format!(r"(?m)^{}:.*$", regex::escape(key)))?;
                let new_fm = re.replace(fm_content, &format!("{}: {}", key, value));
                content = format!("---\n{}{}", new_fm, &content[fm_end..]);
            } else {
                let new_fm = format!("{}\n{}: {}", fm_content, key, value);
                content = format!("---\n{}{}", new_fm, &content[fm_end..]);
            }
        }
    } else {
        content = format!("---\n{}: {}\n---\n\n{}", key, value, content);
    }

    std::fs::write(&path, &content)?;
    println!("{} Set {}: {}", "✓".green().bold(), key.cyan(), value);
    Ok(())
}

fn cmd_remove(vault: &Vault, note_path: &str, key: &str) -> Result<()> {
    let path = fs::resolve_note(vault, note_path)?;
    let content = std::fs::read_to_string(&path)?;
    let note = Note::parse_str(&path, &content)?;

    if !note.frontmatter.contains_key(key) {
        println!("Property '{}' not found.", key);
        return Ok(());
    }

    let mut new_fm = String::new();
    for (k, v) in &note.frontmatter {
        if k != key {
            new_fm.push_str(&format!("{}: {}\n", k, v.to_display_string()));
        }
    }

    let new_content = if new_fm.is_empty() {
        note.body.clone()
    } else {
        format!("---\n{}---\n\n{}", new_fm, note.body)
    };

    std::fs::write(&path, &new_content)?;
    println!("{} Removed property: {}", "✓".green().bold(), key.cyan());
    Ok(())
}
