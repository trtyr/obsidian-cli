//! Batch operations plugin.

use anyhow::Result;
use colored::Colorize;
use regex::Regex;
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::cli::BatchAction;
use crate::kernel::fs;
use crate::kernel::note::Note;
use crate::kernel::vault::Vault;

/// Dispatch batch actions.
pub fn handle(vault: &Vault, action: BatchAction) -> Result<()> {
    match action {
        BatchAction::Rename {
            pattern,
            replacement,
            regex,
            dry_run,
            force,
        } => cmd_batch_rename(vault, &pattern, &replacement, regex, dry_run, force),
        BatchAction::Move {
            pattern,
            destination,
            regex,
            dry_run,
            force,
        } => cmd_batch_move(vault, &pattern, &destination, regex, dry_run, force),
        BatchAction::Delete {
            pattern,
            regex,
            dry_run,
            force,
            permanent,
        } => cmd_batch_delete(vault, &pattern, regex, dry_run, force, permanent),
        BatchAction::Tag {
            pattern,
            tags,
            regex,
            dry_run,
        } => cmd_batch_tag(vault, &pattern, &tags, regex, dry_run),
        BatchAction::Untag {
            pattern,
            tags,
            regex,
            dry_run,
        } => cmd_batch_untag(vault, &pattern, &tags, regex, dry_run),
        BatchAction::Prop {
            pattern,
            key,
            value,
            regex,
            dry_run,
        } => cmd_batch_prop_set(vault, &pattern, &key, &value, regex, dry_run),
        BatchAction::Replace {
            note_pattern,
            find,
            replace,
            regex,
            case_sensitive,
            dry_run,
            force,
        } => cmd_batch_replace(
            vault,
            &note_pattern,
            &find,
            &replace,
            regex,
            case_sensitive,
            dry_run,
            force,
        ),
        BatchAction::Frontmatter {
            pattern,
            properties,
            regex,
            dry_run,
        } => cmd_batch_frontmatter(vault, &pattern, &properties, regex, dry_run),
    }
}

/// Find notes matching a pattern (glob or regex).
fn find_matching_notes(vault: &Vault, pattern: &str, use_regex: bool) -> Result<Vec<PathBuf>> {
    let mut matches = Vec::new();

    if use_regex {
        let re = Regex::new(pattern)?;
        for entry in WalkDir::new(&vault.root)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| !fs::is_hidden_entry(e))
        {
            let entry = entry?;
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
            let rel_path = vault.relative_path(path);
            if re.is_match(&rel_path) {
                matches.push(path.to_path_buf());
            }
        }
    } else {
        let glob_pattern = if pattern.contains('/') {
            vault
                .root
                .join(pattern)
                .to_string_lossy()
                .to_string()
        } else {
            vault
                .root
                .join("**")
                .join(pattern)
                .to_string_lossy()
                .to_string()
        };

        for entry in glob::glob(&glob_pattern)? {
            let path = entry?;
            if path.extension().map_or(true, |e| e != "md") {
                continue;
            }
            if path.starts_with(vault.root.join(".obsidian")) {
                continue;
            }
            matches.push(path);
        }
    }

    matches.sort();
    Ok(matches)
}

fn confirm_batch(operation: &str, count: usize, dry_run: bool) -> Result<bool> {
    if dry_run {
        return Ok(false);
    }
    eprintln!("\n{} {} note(s)? [y/N]", operation.yellow(), count);
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().eq_ignore_ascii_case("y"))
}

fn cmd_batch_rename(
    vault: &Vault,
    pattern: &str,
    replacement: &str,
    use_regex: bool,
    dry_run: bool,
    force: bool,
) -> Result<()> {
    let notes = find_matching_notes(vault, pattern, use_regex)?;
    if notes.is_empty() {
        println!("No notes matched the pattern.");
        return Ok(());
    }

    println!("{} note(s) matched:", notes.len());
    let mut rename_pairs: Vec<(PathBuf, PathBuf)> = Vec::new();

    for path in &notes {
        let name = Vault::note_name(path);
        let new_name = if use_regex {
            let re = Regex::new(pattern)?;
            re.replace(&name, replacement).to_string()
        } else {
            name.replace(pattern, replacement)
        };

        if new_name != name {
            let new_path = path.parent().unwrap().join(format!("{}.md", new_name));
            println!("  {} → {}", name.dimmed(), new_name.green());
            rename_pairs.push((path.clone(), new_path));
        }
    }

    if rename_pairs.is_empty() {
        println!("\nNo renames needed.");
        return Ok(());
    }

    if !force && !confirm_batch("Rename", rename_pairs.len(), dry_run)? {
        println!("Cancelled.");
        return Ok(());
    }

    if dry_run {
        println!(
            "\n{} Dry run complete. Use --force to apply.",
            "✓".green()
        );
        return Ok(());
    }

    let mut success = 0;
    for (old_path, new_path) in &rename_pairs {
        if new_path.exists() {
            eprintln!(
                "  {} Destination exists: {}",
                "✗".red(),
                new_path.display()
            );
            continue;
        }
        std::fs::rename(old_path, new_path)?;
        if vault.app.always_update_links {
            fs::update_links_after_move(vault, old_path, new_path)?;
        }
        success += 1;
    }

    println!("\n{} Renamed {} note(s).", "✓".green().bold(), success);
    Ok(())
}

fn cmd_batch_move(
    vault: &Vault,
    pattern: &str,
    destination: &str,
    use_regex: bool,
    dry_run: bool,
    force: bool,
) -> Result<()> {
    let notes = find_matching_notes(vault, pattern, use_regex)?;
    if notes.is_empty() {
        println!("No notes matched the pattern.");
        return Ok(());
    }

    let dest_dir = vault.root.join(destination);
    if !dry_run && !dest_dir.exists() {
        std::fs::create_dir_all(&dest_dir)?;
    }

    println!("{} note(s) matched:", notes.len());
    for path in &notes {
        let rel = vault.relative_path(path);
        let filename = path.file_name().unwrap();
        let new_path = dest_dir.join(filename);
        println!(
            "  {} → {}",
            rel.dimmed(),
            vault.relative_path(&new_path).green()
        );
    }

    if !force && !confirm_batch("Move", notes.len(), dry_run)? {
        println!("Cancelled.");
        return Ok(());
    }

    if dry_run {
        println!(
            "\n{} Dry run complete. Use --force to apply.",
            "✓".green()
        );
        return Ok(());
    }

    let mut success = 0;
    for path in &notes {
        let filename = path.file_name().unwrap();
        let new_path = dest_dir.join(filename);
        if new_path.exists() {
            eprintln!(
                "  {} Destination exists: {}",
                "✗".red(),
                new_path.display()
            );
            continue;
        }
        std::fs::rename(path, &new_path)?;
        if vault.app.always_update_links {
            fs::update_links_after_move(vault, path, &new_path)?;
        }
        success += 1;
    }

    println!("\n{} Moved {} note(s).", "✓".green().bold(), success);
    Ok(())
}

fn cmd_batch_delete(
    vault: &Vault,
    pattern: &str,
    use_regex: bool,
    dry_run: bool,
    force: bool,
    permanent: bool,
) -> Result<()> {
    let notes = find_matching_notes(vault, pattern, use_regex)?;
    if notes.is_empty() {
        println!("No notes matched the pattern.");
        return Ok(());
    }

    println!("{} note(s) matched:", notes.len());
    for path in &notes {
        println!("  {}", vault.relative_path(path).dimmed());
    }

    if !force && !confirm_batch("Delete", notes.len(), dry_run)? {
        println!("Cancelled.");
        return Ok(());
    }

    if dry_run {
        println!(
            "\n{} Dry run complete. Use --force to apply.",
            "✓".green()
        );
        return Ok(());
    }

    let mut success = 0;
    for path in &notes {
        if permanent {
            std::fs::remove_file(path)?;
        } else {
            #[cfg(target_os = "macos")]
            {
                let trash_dir = dirs::home_dir()
                    .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
                    .join(".Trash");
                let filename = path
                    .file_name()
                    .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;
                let dest = trash_dir.join(filename);
                std::fs::rename(path, &dest)?;
            }
            #[cfg(not(target_os = "macos"))]
            {
                std::fs::remove_file(path)?;
            }
        }
        success += 1;
    }

    println!(
        "\n{} {} {} note(s).",
        "✓".green().bold(),
        if permanent {
            "Permanently deleted"
        } else {
            "Moved to trash"
        },
        success
    );
    Ok(())
}

fn cmd_batch_tag(
    vault: &Vault,
    pattern: &str,
    tags: &[String],
    use_regex: bool,
    dry_run: bool,
) -> Result<()> {
    let notes = find_matching_notes(vault, pattern, use_regex)?;
    if notes.is_empty() {
        println!("No notes matched the pattern.");
        return Ok(());
    }

    println!("{} note(s) matched:", notes.len());
    for path in &notes {
        println!("  {} +[{}]", vault.relative_path(path).dimmed(), tags.join(", "));
    }

    if dry_run {
        println!("\n{} Dry run complete.", "✓".green());
        return Ok(());
    }

    let mut success = 0;
    for path in &notes {
        let content = std::fs::read_to_string(path)?;
        let note = Note::parse_str(path, &content)?;
        let mut existing_tags = note.property_tags();

        let mut added = false;
        for tag in tags {
            if !existing_tags.contains(tag) {
                existing_tags.push(tag.clone());
                added = true;
            }
        }

        if !added {
            continue;
        }

        let new_content = rebuild_frontmatter_with_tags(&content, &existing_tags)?;
        std::fs::write(path, &new_content)?;
        success += 1;
    }

    println!("\n{} Tagged {} note(s).", "✓".green().bold(), success);
    Ok(())
}

fn cmd_batch_untag(
    vault: &Vault,
    pattern: &str,
    tags: &[String],
    use_regex: bool,
    dry_run: bool,
) -> Result<()> {
    let notes = find_matching_notes(vault, pattern, use_regex)?;
    if notes.is_empty() {
        println!("No notes matched the pattern.");
        return Ok(());
    }

    println!("{} note(s) matched:", notes.len());
    for path in &notes {
        println!("  {} -[{}]", vault.relative_path(path).dimmed(), tags.join(", "));
    }

    if dry_run {
        println!("\n{} Dry run complete.", "✓".green());
        return Ok(());
    }

    let mut success = 0;
    for path in &notes {
        let content = std::fs::read_to_string(path)?;
        let note = Note::parse_str(path, &content)?;
        let existing_tags = note.property_tags();
        let new_tags: Vec<String> = existing_tags
            .iter()
            .filter(|t| !tags.contains(t))
            .cloned()
            .collect();

        if new_tags.len() == existing_tags.len() {
            continue;
        }

        let new_content = rebuild_frontmatter_with_tags(&content, &new_tags)?;
        std::fs::write(path, &new_content)?;
        success += 1;
    }

    println!("\n{} Untagged {} note(s).", "✓".green().bold(), success);
    Ok(())
}

fn cmd_batch_prop_set(
    vault: &Vault,
    pattern: &str,
    key: &str,
    value: &str,
    use_regex: bool,
    dry_run: bool,
) -> Result<()> {
    let notes = find_matching_notes(vault, pattern, use_regex)?;
    if notes.is_empty() {
        println!("No notes matched the pattern.");
        return Ok(());
    }

    println!("{} note(s) matched:", notes.len());
    for path in &notes {
        println!(
            "  {} {}: {}",
            vault.relative_path(path).dimmed(),
            key.cyan(),
            value
        );
    }

    if dry_run {
        println!("\n{} Dry run complete.", "✓".green());
        return Ok(());
    }

    let mut success = 0;
    for path in &notes {
        let content = std::fs::read_to_string(path)?;
        let mut new_content = content.clone();

        if content.starts_with("---") {
            if let Some(end) = content[3..].find("\n---") {
                let fm_end = 3 + end;
                let fm_content = &content[3..fm_end];

                let key_pattern = format!("{}:", key);
                if fm_content.contains(&key_pattern) {
                    let re = Regex::new(&format!(r"(?m)^{}:.*$", regex::escape(key)))?;
                    let new_fm = re.replace(fm_content, &format!("{}: {}", key, value));
                    new_content = format!("---\n{}{}", new_fm, &content[fm_end..]);
                } else {
                    let new_fm = format!("{}\n{}: {}", fm_content, key, value);
                    new_content = format!("---\n{}{}", new_fm, &content[fm_end..]);
                }
            }
        } else {
            new_content = format!("---\n{}: {}\n---\n\n{}", key, value, content);
        }

        std::fs::write(path, &new_content)?;
        success += 1;
    }

    println!(
        "\n{} Updated {} note(s).",
        "✓".green().bold(),
        success
    );
    Ok(())
}

fn cmd_batch_replace(
    vault: &Vault,
    note_pattern: &str,
    find: &str,
    replace: &str,
    use_regex: bool,
    case_sensitive: bool,
    dry_run: bool,
    force: bool,
) -> Result<()> {
    let notes = find_matching_notes(vault, note_pattern, use_regex)?;
    if notes.is_empty() {
        println!("No notes matched the pattern.");
        return Ok(());
    }

    let search_pattern = if use_regex {
        let flags = if case_sensitive { "" } else { "(?i)" };
        Regex::new(&format!("{}{}", flags, find))?
    } else {
        let escaped = regex::escape(find);
        let flags = if case_sensitive { "" } else { "(?i)" };
        Regex::new(&format!("{}{}", flags, escaped))?
    };

    println!("{} note(s) matched:", notes.len());
    let mut replace_pairs: Vec<(PathBuf, String, usize)> = Vec::new();

    for path in &notes {
        let content = std::fs::read_to_string(path)?;
        let count = search_pattern.find_iter(&content).count();
        if count > 0 {
            println!(
                "  {} ({} occurrence(s))",
                vault.relative_path(path).dimmed(),
                count
            );
            let new_content = search_pattern.replace_all(&content, replace).to_string();
            replace_pairs.push((path.clone(), new_content, count));
        }
    }

    if replace_pairs.is_empty() {
        println!("\nNo replacements needed.");
        return Ok(());
    }

    let total: usize = replace_pairs.iter().map(|(_, _, c)| c).sum();
    if !force
        && !confirm_batch(
            &format!("Replace {} occurrence(s) in", total),
            replace_pairs.len(),
            dry_run,
        )?
    {
        println!("Cancelled.");
        return Ok(());
    }

    if dry_run {
        println!(
            "\n{} Dry run complete. Use --force to apply.",
            "✓".green()
        );
        return Ok(());
    }

    let mut success = 0;
    for (path, new_content, _) in &replace_pairs {
        std::fs::write(path, new_content)?;
        success += 1;
    }

    println!(
        "\n{} Replaced {} occurrence(s) in {} note(s).",
        "✓".green().bold(),
        total,
        success
    );
    Ok(())
}

fn cmd_batch_frontmatter(
    vault: &Vault,
    pattern: &str,
    properties: &[String],
    use_regex: bool,
    dry_run: bool,
) -> Result<()> {
    let notes = find_matching_notes(vault, pattern, use_regex)?;
    if notes.is_empty() {
        println!("No notes matched the pattern.");
        return Ok(());
    }

    let mut prop_map: HashMap<String, String> = HashMap::new();
    for prop in properties {
        if let Some((k, v)) = prop.split_once('=') {
            prop_map.insert(k.to_string(), v.to_string());
        } else {
            anyhow::bail!("Invalid property format: {} (expected key=value)", prop);
        }
    }

    println!("{} note(s) matched:", notes.len());
    for path in &notes {
        println!("  {} +{:?}", vault.relative_path(path).dimmed(), prop_map);
    }

    if dry_run {
        println!("\n{} Dry run complete.", "✓".green());
        return Ok(());
    }

    let mut success = 0;
    for path in &notes {
        let content = std::fs::read_to_string(path)?;

        let new_content = if content.starts_with("---") {
            if let Some(end) = content[3..].find("\n---") {
                let fm_end = 3 + end;
                let fm_content = &content[3..fm_end];
                let mut new_fm = fm_content.to_string();

                for (key, value) in &prop_map {
                    let key_pattern = format!("{}:", key);
                    if fm_content.contains(&key_pattern) {
                        let re =
                            Regex::new(&format!(r"(?m)^{}:.*$", regex::escape(key)))?;
                        new_fm = re
                            .replace(&new_fm, &format!("{}: {}", key, value))
                            .to_string();
                    } else {
                        new_fm = format!("{}\n{}: {}", new_fm, key, value);
                    }
                }

                format!("---\n{}{}", new_fm, &content[fm_end..])
            } else {
                content.clone()
            }
        } else {
            let mut fm = String::new();
            for (key, value) in &prop_map {
                fm.push_str(&format!("{}: {}\n", key, value));
            }
            format!("---\n{}---\n\n{}", fm, content)
        };

        std::fs::write(path, &new_content)?;
        success += 1;
    }

    println!(
        "\n{} Updated {} note(s).",
        "✓".green().bold(),
        success
    );
    Ok(())
}

/// Rebuild frontmatter with updated tags.
fn rebuild_frontmatter_with_tags(content: &str, tags: &[String]) -> Result<String> {
    if content.starts_with("---") {
        if let Some(end) = content[3..].find("\n---") {
            let fm_end = 3 + end;
            let fm_content = &content[3..fm_end];
            let body = &content[fm_end..];

            let mut lines: Vec<String> = fm_content.lines().map(|l| l.to_string()).collect();

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

    let mut fm = "tags:\n".to_string();
    for tag in tags {
        fm.push_str(&format!("  - {}\n", tag));
    }
    Ok(format!("---\n{}---\n\n{}", fm, content))
}
