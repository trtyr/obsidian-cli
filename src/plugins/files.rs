//! Note CRUD plugin — handles all note operations.

use anyhow::{Context, Result};
use chrono::Local;
use colored::Colorize;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::cli::NoteAction;
use crate::kernel::fs;
use crate::kernel::index::VaultIndex;
use crate::kernel::note::Note;
use crate::kernel::output;
use crate::kernel::vault::Vault;

/// Dispatch note actions.
pub fn handle(vault: &Vault, action: NoteAction) -> Result<()> {
    match action {
        NoteAction::Create {
            name,
            content,
            template,
            tags,
            open,
        } => cmd_create(vault, &name, content, template, tags, open),
        NoteAction::Read {
            note,
            frontmatter_only,
            body_only,
        } => cmd_read(vault, &note, frontmatter_only, body_only),
        NoteAction::Edit { note } => cmd_edit(vault, &note),
        NoteAction::Delete {
            note,
            force,
            permanent,
        } => cmd_delete(vault, &note, force, permanent),
        NoteAction::Move {
            source,
            destination,
            force,
        } => cmd_move(vault, &source, &destination, force),
        NoteAction::Copy {
            source,
            destination,
        } => cmd_copy(vault, &source, &destination),
        NoteAction::List {
            path,
            recursive,
            extension,
        } => cmd_list(vault, path, recursive, &extension),
        NoteAction::Append { note, content } => cmd_append(vault, &note, &content),
        NoteAction::Prepend { note, content } => cmd_prepend(vault, &note, &content),
        NoteAction::Stats { note } => cmd_stats(vault, note),
        NoteAction::Aliases { note } => cmd_aliases(vault, &note),
        NoteAction::Merge {
            sources,
            destination,
            separator,
            delete_sources,
        } => cmd_merge(vault, sources, &destination, &separator, delete_sources),
        NoteAction::Split {
            note,
            level,
            output,
            delete_source,
        } => cmd_split(vault, &note, level, output, delete_source),
    }
}

fn cmd_create(
    vault: &Vault,
    note_path: &str,
    content: Option<String>,
    template: Option<String>,
    tags: Vec<String>,
    open: bool,
) -> Result<()> {
    let path = if Path::new(note_path).is_absolute() {
        PathBuf::from(note_path)
    } else {
        let p = vault.root.join(note_path);
        if p.extension().is_none() {
            p.with_extension("md")
        } else {
            p
        }
    };

    if path.exists() {
        anyhow::bail!("Note already exists: {}", path.display());
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut note_content = String::new();

    // Apply template
    if let Some(ref tmpl) = template {
        let tmpl_path = fs::find_template(vault, tmpl)?;
        let tmpl_content = std::fs::read_to_string(&tmpl_path)?;
        let name = Vault::note_name(&path);
        let date = Local::now().format("%Y-%m-%d").to_string();
        let datetime = Local::now().format("%Y-%m-%d %H:%M").to_string();

        note_content = tmpl_content
            .replace("{{title}}", &name)
            .replace("{{date}}", &date)
            .replace("{{datetime}}", &datetime)
            .replace("{{name}}", &name);
    }

    // Add tags
    if !tags.is_empty() {
        if note_content.contains("---") {
            note_content = note_content.replacen(
                "---",
                &format!(
                    "---\ntags:\n{}\n",
                    tags.iter()
                        .map(|t| format!("  - {}", t))
                        .collect::<Vec<_>>()
                        .join("\n")
                ),
                1,
            );
        } else {
            let fm = format!(
                "---\ntags:\n{}\ncreated: {}\n---\n\n",
                tags.iter()
                    .map(|t| format!("  - {}", t))
                    .collect::<Vec<_>>()
                    .join("\n"),
                Local::now().format("%Y-%m-%d %H:%M")
            );
            note_content = format!("{}{}", fm, note_content);
        }
    }

    // Append content
    if let Some(ref c) = content {
        note_content.push_str(c);
    } else if note_content.is_empty() {
        let mut buffer = String::new();
        if !is_terminal::is_terminal(io::stdin()) {
            io::stdin().read_to_string(&mut buffer)?;
            note_content = buffer;
        }
    }

    if !note_content.ends_with('\n') {
        note_content.push('\n');
    }

    std::fs::write(&path, &note_content)?;
    println!("{} Created: {}", "✓".green().bold(), path.display());

    if open {
        fs::open_in_editor(&path)?;
    }

    Ok(())
}

fn cmd_read(
    vault: &Vault,
    note_path: &str,
    frontmatter_only: bool,
    body_only: bool,
) -> Result<()> {
    let path = fs::resolve_note(vault, note_path)?;
    let note = Note::parse(&path)?;

    if frontmatter_only {
        if note.frontmatter.is_empty() {
            println!("No frontmatter.");
        } else {
            for (k, v) in &note.frontmatter {
                println!("{}: {}", k.cyan(), v.to_display_string());
            }
        }
    } else if body_only {
        print!("{}", note.body);
    } else {
        print!("{}", note.raw);
    }

    Ok(())
}

fn cmd_edit(vault: &Vault, note_path: &str) -> Result<()> {
    let path = fs::resolve_note(vault, note_path)?;
    fs::open_in_editor(&path)
}

fn cmd_delete(vault: &Vault, note_path: &str, force: bool, permanent: bool) -> Result<()> {
    let path = fs::resolve_note(vault, note_path)?;

    if !force {
        eprintln!("{} Delete {}? [y/N]", "⚠".yellow(), path.display());
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    if permanent {
        std::fs::remove_file(&path)?;
        println!(
            "{} Permanently deleted: {}",
            "✓".green().bold(),
            path.display()
        );
    } else {
        #[cfg(target_os = "macos")]
        {
            let trash_dir = dirs::home_dir()
                .context("Could not find home directory")?
                .join(".Trash");
            let filename = path.file_name().context("Invalid filename")?;
            let dest = trash_dir.join(filename);
            std::fs::rename(&path, &dest)?;
            println!("{} Moved to trash: {}", "✓".green().bold(), path.display());
        }
        #[cfg(not(target_os = "macos"))]
        {
            std::fs::remove_file(&path)?;
            println!("{} Deleted: {}", "✓".green().bold(), path.display());
        }
    }

    Ok(())
}

fn cmd_move(vault: &Vault, source: &str, destination: &str, force: bool) -> Result<()> {
    let src_path = fs::resolve_note(vault, source)?;
    let dest_path = if Path::new(destination).is_absolute() {
        PathBuf::from(destination)
    } else {
        vault.root.join(destination)
    };

    let dest_path = if dest_path.extension().is_none() {
        dest_path.with_extension("md")
    } else {
        dest_path
    };

    if dest_path.exists() && !force {
        anyhow::bail!("Destination already exists: {}", dest_path.display());
    }

    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::rename(&src_path, &dest_path)?;

    if vault.app.always_update_links {
        fs::update_links_after_move(vault, &src_path, &dest_path)?;
    }

    println!(
        "{} Moved: {} → {}",
        "✓".green().bold(),
        vault.relative_path(&src_path),
        vault.relative_path(&dest_path)
    );

    Ok(())
}

fn cmd_copy(vault: &Vault, source: &str, destination: &str) -> Result<()> {
    let src_path = fs::resolve_note(vault, source)?;
    let dest_path = if Path::new(destination).is_absolute() {
        PathBuf::from(destination)
    } else {
        vault.root.join(destination)
    };

    let dest_path = if dest_path.extension().is_none() {
        dest_path.with_extension("md")
    } else {
        dest_path
    };

    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::copy(&src_path, &dest_path)?;
    println!(
        "{} Copied: {} → {}",
        "✓".green().bold(),
        vault.relative_path(&src_path),
        vault.relative_path(&dest_path)
    );

    Ok(())
}

fn cmd_list(vault: &Vault, path: Option<String>, recursive: bool, ext: &str) -> Result<()> {
    let dir = match path {
        Some(p) => vault.resolve_path(&p),
        None => vault.root.clone(),
    };

    if !dir.exists() {
        anyhow::bail!("Directory not found: {}", dir.display());
    }

    let walker = if recursive {
        WalkDir::new(&dir).follow_links(true)
    } else {
        WalkDir::new(&dir).max_depth(1).follow_links(true)
    };

    let mut files: Vec<PathBuf> = Vec::new();
    for entry in walker.into_iter().filter_entry(|e| !fs::is_hidden_entry(e)) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let p = entry.path();
            if p.extension().map_or(false, |e| e == ext) {
                files.push(p.to_path_buf());
            }
        }
    }

    files.sort();
    for f in &files {
        let rel = vault.relative_path(f);
        println!("{}", rel);
    }

    eprintln!("\n{} file(s)", files.len());
    Ok(())
}

fn cmd_append(vault: &Vault, note_path: &str, content: &str) -> Result<()> {
    let path = fs::resolve_note(vault, note_path)?;
    let mut file_content = std::fs::read_to_string(&path)?;

    if !file_content.ends_with('\n') {
        file_content.push('\n');
    }
    file_content.push_str(content);
    if !content.ends_with('\n') {
        file_content.push('\n');
    }

    std::fs::write(&path, &file_content)?;
    println!("{} Appended to: {}", "✓".green().bold(), path.display());
    Ok(())
}

fn cmd_prepend(vault: &Vault, note_path: &str, content: &str) -> Result<()> {
    let path = fs::resolve_note(vault, note_path)?;
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
    println!("{} Prepended to: {}", "✓".green().bold(), path.display());
    Ok(())
}

fn cmd_stats(vault: &Vault, note: Option<String>) -> Result<()> {
    match note {
        Some(n) => {
            let path = fs::resolve_note(vault, &n)?;
            let note = Note::parse(&path)?;
            println!("{}", output::format_wordcount(&note));
        }
        None => {
            let index = VaultIndex::build(vault)?;
            let total_notes = index.notes.len();
            let total_words: usize = index.notes.values().map(|n| n.word_count).sum();
            let total_links: usize = index.notes.values().map(|n| n.wikilinks.len()).sum();
            let total_tags = index.tag_index.len();
            let total_tasks: usize = index.notes.values().map(|n| n.tasks.len()).sum();

            println!("{}", "Vault Statistics:".bold());
            println!("  {} {}", "Notes:".dimmed(), total_notes);
            println!("  {} {}", "Words:".dimmed(), total_words);
            println!("  {} {}", "Links:".dimmed(), total_links);
            println!("  {} {}", "Tags:".dimmed(), total_tags);
            println!("  {} {}", "Tasks:".dimmed(), total_tasks);
        }
    }
    Ok(())
}

fn cmd_aliases(vault: &Vault, note_path: &str) -> Result<()> {
    let path = fs::resolve_note(vault, note_path)?;
    let note = Note::parse(&path)?;

    let aliases = note.aliases();
    if aliases.is_empty() {
        println!("No aliases for {}", note.name);
    } else {
        println!("Aliases for {}:", note.name.bold());
        for alias in &aliases {
            println!("  {}", alias);
        }
    }
    Ok(())
}

/// Merge multiple notes into one.
fn cmd_merge(
    vault: &Vault,
    sources: Vec<String>,
    destination: &str,
    separator: &str,
    delete_sources: bool,
) -> Result<()> {
    let mut merged_content = String::new();
    let mut source_paths = Vec::new();

    for (i, source) in sources.iter().enumerate() {
        let path = fs::resolve_note(vault, source)?;
        let content = std::fs::read_to_string(&path)?;

        if i > 0 {
            merged_content.push_str(separator);
        }
        merged_content.push_str(&content);
        source_paths.push(path);
    }

    // Write merged content
    let dest_path = if Path::new(destination).is_absolute() {
        PathBuf::from(destination)
    } else {
        let p = vault.root.join(destination);
        if p.extension().is_none() {
            p.with_extension("md")
        } else {
            p
        }
    };

    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(&dest_path, &merged_content)?;
    println!("{} Merged {} notes into {}", "✓".green().bold(), sources.len(), dest_path.display());

    // Delete source notes if requested
    if delete_sources {
        for path in &source_paths {
            std::fs::remove_file(path)?;
            println!("  {} Deleted source: {}", "✓".green(), vault.relative_path(path));
        }
    }

    Ok(())
}

/// Split a note by headings into separate notes.
fn cmd_split(
    vault: &Vault,
    note_path: &str,
    level: usize,
    output: Option<String>,
    delete_source: bool,
) -> Result<()> {
    let path = fs::resolve_note(vault, note_path)?;
    let content = std::fs::read_to_string(&path)?;

    let output_dir = match output {
        Some(dir) => {
            let p = vault.root.join(&dir);
            std::fs::create_dir_all(&p)?;
            p
        }
        None => path.parent()
            .ok_or_else(|| anyhow::anyhow!("Could not determine parent directory"))?
            .to_path_buf(),
    };

    // Extract frontmatter if present
    let frontmatter = if content.starts_with("---") {
        if let Some(end) = content[3..].find("\n---") {
            Some(&content[..3 + end + 4])
        } else {
            None
        }
    } else {
        None
    };

    // Split by headings
    let heading_prefix = "#".repeat(level);
    let mut sections: Vec<(String, String)> = Vec::new();
    let mut current_title = "untitled".to_string();
    let mut current_content = String::new();

    for line in content.lines() {
        if line.starts_with(&heading_prefix) && line.chars().nth(level) == Some(' ') {
            // Save previous section
            if !current_content.trim().is_empty() || current_title != "untitled" {
                sections.push((current_title.clone(), current_content.clone()));
            }
            // Start new section
            current_title = line[level + 1..].trim().to_string();
            current_content = String::new();
            // Include frontmatter in each section
            if let Some(fm) = frontmatter {
                current_content.push_str(fm);
                current_content.push('\n');
            }
        } else {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    // Save last section
    if !current_content.trim().is_empty() || current_title != "untitled" {
        sections.push((current_title, current_content));
    }

    if sections.is_empty() {
        println!("{}", "No sections found to split.".yellow());
        return Ok(());
    }

    // Write sections to files
    println!("Splitting into {} section(s):", sections.len());
    for (title, content) in &sections {
        let safe_name = title
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' { c } else { '_' })
            .collect::<String>()
            .replace(' ', "-");
        let dest_path = output_dir.join(format!("{}.md", safe_name));

        std::fs::write(&dest_path, content)?;
        println!("  {} {}", "✓".green(), dest_path.display());
    }

    // Delete source if requested
    if delete_source {
        std::fs::remove_file(&path)?;
        println!("  {} Deleted source: {}", "✓".green(), vault.relative_path(&path));
    }

    Ok(())
}
