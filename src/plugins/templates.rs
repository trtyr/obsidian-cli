//! Template operations plugin — list, read, create, delete, apply.

use anyhow::Result;
use chrono::Local;
use colored::Colorize;
use std::collections::HashMap;

use crate::cli::TemplateAction;
use crate::kernel::fs;
use crate::kernel::vault::Vault;

/// Dispatch template actions.
pub fn handle(vault: &Vault, action: TemplateAction) -> Result<()> {
    match action {
        TemplateAction::List => cmd_list(vault),
        TemplateAction::Read { name } => cmd_read(vault, &name),
        TemplateAction::Create { note, name } => cmd_create(vault, &note, &name),
        TemplateAction::Delete { name, force } => cmd_delete(vault, &name, force),
        TemplateAction::Apply {
            template,
            note,
            vars,
        } => cmd_apply(vault, &template, &note, vars),
    }
}

fn cmd_list(vault: &Vault) -> Result<()> {
    let tmpl_dir = fs::find_template_dir(vault)?;
    let mut templates: Vec<String> = Vec::new();

    for entry in std::fs::read_dir(&tmpl_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "md") {
            templates.push(Vault::note_name(&path));
        }
    }

    templates.sort();

    if templates.is_empty() {
        println!("{}", "No templates found.".dimmed());
    } else {
        println!("{} template(s):", templates.len());
        for t in &templates {
            println!("  {}", t);
        }
    }
    Ok(())
}

fn cmd_read(vault: &Vault, name: &str) -> Result<()> {
    let path = fs::find_template(vault, name)?;
    let content = std::fs::read_to_string(&path)?;
    print!("{}", content);
    Ok(())
}

fn cmd_create(vault: &Vault, note_path: &str, template_name: &str) -> Result<()> {
    let note_path_resolved = fs::resolve_note(vault, note_path)?;
    let content = std::fs::read_to_string(&note_path_resolved)?;

    let tmpl_dir = fs::find_template_dir(vault)?;
    let dest = tmpl_dir.join(format!("{}.md", template_name));

    if dest.exists() {
        anyhow::bail!("Template already exists: {}", dest.display());
    }

    std::fs::write(&dest, &content)?;
    println!(
        "{} Created template '{}' from {}",
        "✓".green().bold(),
        template_name,
        note_path
    );
    Ok(())
}

fn cmd_delete(vault: &Vault, name: &str, force: bool) -> Result<()> {
    let path = fs::find_template(vault, name)?;

    if !force {
        eprintln!("{} Delete template {}? [y/N]", "⚠".yellow(), name);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    std::fs::remove_file(&path)?;
    println!("{} Deleted template: {}", "✓".green().bold(), name);
    Ok(())
}

fn cmd_apply(vault: &Vault, template: &str, note: &str, vars: Vec<String>) -> Result<()> {
    let tmpl_path = fs::find_template(vault, template)?;
    let mut content = std::fs::read_to_string(&tmpl_path)?;

    // Parse variables
    let mut var_map: HashMap<String, String> = HashMap::new();
    for var in &vars {
        if let Some((k, v)) = var.split_once('=') {
            var_map.insert(k.to_string(), v.to_string());
        }
    }

    // Built-in variables
    let name = if note.contains('/') {
        std::path::Path::new(note)
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string()
    } else {
        note.to_string()
    };
    var_map
        .entry("title".to_string())
        .or_insert_with(|| name.clone());
    var_map
        .entry("name".to_string())
        .or_insert_with(|| name.clone());
    var_map
        .entry("date".to_string())
        .or_insert_with(|| Local::now().format("%Y-%m-%d").to_string());
    var_map
        .entry("datetime".to_string())
        .or_insert_with(|| Local::now().format("%Y-%m-%d %H:%M").to_string());

    // Apply substitutions
    for (key, value) in &var_map {
        content = content.replace(&format!("{{{{{}}}}}", key), value);
    }

    // Create the note
    let path = vault.root.join(note);
    let path = if path.extension().is_none() {
        path.with_extension("md")
    } else {
        path
    };

    if path.exists() {
        anyhow::bail!("Note already exists: {}", path.display());
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(&path, &content)?;
    println!(
        "{} Created from template: {}",
        "✓".green().bold(),
        path.display()
    );
    Ok(())
}
