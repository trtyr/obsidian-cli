//! Filesystem helpers shared across plugins.

use anyhow::Result;
use std::path::{Path, PathBuf};

use super::index::VaultIndex;
use super::vault::Vault;

/// Resolve a note path from user input.
///
/// Tries: exact path → add .md → fuzzy name lookup.
pub fn resolve_note(vault: &Vault, input: &str) -> Result<PathBuf> {
    let path = vault.resolve_path(input);

    if path.exists() {
        return Ok(path);
    }

    // Try adding .md extension
    let with_ext = if path.extension().is_none() {
        path.with_extension("md")
    } else {
        path.clone()
    };

    if with_ext.exists() {
        return Ok(with_ext);
    }

    // Try searching by name
    let index = VaultIndex::build(vault)?;
    if let Some(note) = index.find_by_name(input) {
        return Ok(note.path.clone());
    }

    anyhow::bail!("Note not found: {}", input)
}

/// Open a file in the user's editor.
pub fn open_in_editor(path: &Path) -> Result<()> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| {
        if cfg!(target_os = "macos") {
            "open".to_string()
        } else {
            "vi".to_string()
        }
    });

    let status = std::process::Command::new(&editor)
        .arg(path)
        .status()?;

    if !status.success() {
        anyhow::bail!("Editor exited with status: {}", status);
    }

    Ok(())
}

/// Update wikilinks after moving a note.
pub fn update_links_after_move(vault: &Vault, old_path: &Path, new_path: &Path) -> Result<()> {
    let old_name = Vault::note_name(old_path);
    let new_name = Vault::note_name(new_path);

    if old_name == new_name {
        return Ok(());
    }

    let index = VaultIndex::build(vault)?;
    let backlinks = index.get_backlinks(&old_name);

    for note in &backlinks {
        let mut content = std::fs::read_to_string(&note.path)?;
        let old_link = format!("[[{}]]", old_name);
        let new_link = format!("[[{}]]", new_name);
        content = content.replace(&old_link, &new_link);

        let old_link_pipe = format!("[[{}|", old_name);
        let new_link_pipe = format!("[[{}|", new_name);
        content = content.replace(&old_link_pipe, &new_link_pipe);

        std::fs::write(&note.path, &content)?;
    }

    if !backlinks.is_empty() {
        eprintln!("  Updated {} link(s) in other notes", backlinks.len());
    }

    Ok(())
}

/// Check if a walkdir entry is hidden (starts with '.').
pub fn is_hidden_entry(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

/// Find template directory in vault.
pub fn find_template_dir(vault: &Vault) -> Result<PathBuf> {
    let candidates = vec![
        vault.root.join("templates"),
        vault.root.join("Templates"),
        vault.root.join("template"),
        vault.root.join("Template"),
    ];

    for dir in &candidates {
        if dir.exists() {
            return Ok(dir.clone());
        }
    }

    anyhow::bail!(
        "No templates directory found. Create one at {}/templates",
        vault.root.display()
    )
}

/// Find a template by name (exact → fuzzy).
pub fn find_template(vault: &Vault, name: &str) -> Result<PathBuf> {
    let tmpl_dir = find_template_dir(vault)?;

    let exact = tmpl_dir.join(format!("{}.md", name));
    if exact.exists() {
        return Ok(exact);
    }

    let no_ext = tmpl_dir.join(name);
    if no_ext.exists() {
        return Ok(no_ext);
    }

    for entry in std::fs::read_dir(&tmpl_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "md") {
            let file_name = Vault::note_name(&path);
            if file_name.to_lowercase().contains(&name.to_lowercase()) {
                return Ok(path);
            }
        }
    }

    anyhow::bail!("Template not found: {}", name)
}
