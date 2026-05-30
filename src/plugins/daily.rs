//! Daily note operations plugin — today, read, create, append, prepend, path, list.

use anyhow::Result;
use chrono::Local;
use colored::Colorize;
use std::path::PathBuf;

use crate::cli::DailyAction;
use crate::kernel::vault::Vault;

/// Dispatch daily note actions.
pub fn handle(vault: &Vault, action: DailyAction) -> Result<()> {
    match action {
        DailyAction::Today => cmd_read(vault, None),
        DailyAction::Read { date } => cmd_read(vault, date),
        DailyAction::Create { date } => cmd_create(vault, date),
        DailyAction::Append { content } => cmd_append(vault, &content),
        DailyAction::Prepend { content } => cmd_prepend(vault, &content),
        DailyAction::Path { date } => cmd_path(vault, date),
        DailyAction::List => cmd_list(vault),
    }
}

fn cmd_read(vault: &Vault, date: Option<String>) -> Result<()> {
    let date = date.unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string());
    let path = daily_note_path(vault, &date)?;

    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        print!("{}", content);
    } else {
        anyhow::bail!("No daily note for {}", date);
    }
    Ok(())
}

fn cmd_create(vault: &Vault, date: Option<String>) -> Result<()> {
    let date = date.unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string());
    let path = daily_note_path(vault, &date)?;

    if path.exists() {
        println!("Daily note already exists: {}", path.display());
        return Ok(());
    }

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Check for daily note template
    let content = if let Some(ref tmpl_name) = vault.daily.template {
        match crate::kernel::fs::find_template(vault, tmpl_name) {
            Ok(tmpl_path) => {
                let mut tmpl_content = std::fs::read_to_string(&tmpl_path)?;
                tmpl_content = tmpl_content
                    .replace("{{date}}", &date)
                    .replace("{{title}}", &date)
                    .replace("{{datetime}}", &Local::now().format("%Y-%m-%d %H:%M").to_string());
                tmpl_content
            }
            Err(_) => format!("# {}\n\n", date),
        }
    } else {
        format!("# {}\n\n", date)
    };

    std::fs::write(&path, &content)?;
    println!("{} Created daily note: {}", "✓".green().bold(), path.display());
    Ok(())
}

fn cmd_append(vault: &Vault, content: &str) -> Result<()> {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let path = daily_note_path(vault, &today)?;

    if !path.exists() {
        // Auto-create
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, format!("# {}\n\n", today))?;
    }

    let mut file_content = std::fs::read_to_string(&path)?;
    if !file_content.ends_with('\n') {
        file_content.push('\n');
    }
    file_content.push_str(content);
    if !content.ends_with('\n') {
        file_content.push('\n');
    }

    std::fs::write(&path, &file_content)?;
    println!("{} Appended to daily note", "✓".green().bold());
    Ok(())
}

fn cmd_prepend(vault: &Vault, content: &str) -> Result<()> {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let path = daily_note_path(vault, &today)?;

    if !path.exists() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, format!("# {}\n\n", today))?;
    }

    let file_content = std::fs::read_to_string(&path)?;
    let new_content = if file_content.starts_with("---") {
        if let Some(end) = file_content[3..].find("\n---") {
            let insert_pos = 3 + end + 4;
            let mut result = file_content[..insert_pos].to_string();
            result.push('\n');
            result.push_str(content);
            result.push('\n');
            result.push_str(&file_content[insert_pos..]);
            result
        } else {
            format!("{}\n{}", content, file_content)
        }
    } else {
        format!("{}\n{}", content, file_content)
    };

    std::fs::write(&path, &new_content)?;
    println!("{} Prepended to daily note", "✓".green().bold());
    Ok(())
}

fn cmd_path(vault: &Vault, date: Option<String>) -> Result<()> {
    let date = date.unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string());
    let path = daily_note_path(vault, &date)?;
    println!("{}", path.display());
    Ok(())
}

fn cmd_list(vault: &Vault) -> Result<()> {
    let daily_dir = vault.daily_folder();
    if !daily_dir.exists() {
        println!("No daily notes directory found.");
        return Ok(());
    }

    let mut notes: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(&daily_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "md") {
            notes.push(path);
        }
    }

    notes.sort();
    notes.reverse();

    for note in &notes {
        let name = Vault::note_name(note);
        println!("{}", name);
    }

    println!("\n{} daily note(s)", notes.len());
    Ok(())
}

fn daily_note_path(vault: &Vault, date: &str) -> Result<PathBuf> {
    let format = vault.daily.format.as_deref().unwrap_or("YYYY-MM-DD");
    let filename = format
        .replace("YYYY", &date[0..4])
        .replace("MM", &date[5..7])
        .replace("DD", &date[8..10]);

    Ok(vault.daily_folder().join(format!("{}.md", filename)))
}
