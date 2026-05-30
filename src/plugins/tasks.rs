//! Task operations plugin — list, add, done, undone, remove.

use anyhow::Result;
use colored::Colorize;

use crate::cli::TaskAction;
use crate::kernel::fs;
use crate::kernel::index::VaultIndex;
use crate::kernel::note::Note;
use crate::kernel::output;
use crate::kernel::vault::Vault;

/// Dispatch task actions.
pub fn handle(vault: &Vault, action: TaskAction) -> Result<()> {
    match action {
        TaskAction::List {
            note,
            pending,
            done,
            tag,
        } => cmd_list(vault, note, pending, done, tag),
        TaskAction::Add { note, text } => cmd_add(vault, &note, &text),
        TaskAction::Done { note, line } => cmd_toggle(vault, &note, line, true),
        TaskAction::Undone { note, line } => cmd_toggle(vault, &note, line, false),
        TaskAction::Remove { note, line } => cmd_remove(vault, &note, line),
    }
}

fn cmd_list(
    vault: &Vault,
    note: Option<String>,
    pending: bool,
    done: bool,
    tag: Option<String>,
) -> Result<()> {
    match note {
        Some(n) => {
            let path = fs::resolve_note(vault, &n)?;
            let note = Note::parse(&path)?;
            let mut tasks: Vec<_> = note.get_tasks().iter().collect();

            if pending {
                tasks.retain(|t| !t.done);
            } else if done {
                tasks.retain(|t| t.done);
            }
            if let Some(ref t) = tag {
                tasks.retain(|task| task.tags.contains(t));
            }

            println!("{}", output::format_tasks(&tasks));
        }
        None => {
            let index = VaultIndex::build(vault)?;
            let mut all_tasks: Vec<(String, Vec<&crate::kernel::note::Task>)> = Vec::new();

            for note in index.notes.values() {
                let mut tasks: Vec<_> = note.get_tasks().iter().collect();
                if pending {
                    tasks.retain(|t| !t.done);
                } else if done {
                    tasks.retain(|t| t.done);
                }
                if let Some(ref t) = tag {
                    tasks.retain(|task| task.tags.contains(t));
                }
                if !tasks.is_empty() {
                    all_tasks.push((note.name.clone(), tasks));
                }
            }

            let total: usize = all_tasks.iter().map(|(_, t)| t.len()).sum();
            println!("{} task(s) across {} note(s)\n", total, all_tasks.len());

            for (name, tasks) in &all_tasks {
                println!("{}:", name.bold());
                for task in tasks {
                    let checkbox = if task.done {
                        "[x]".green()
                    } else {
                        "[ ]".yellow()
                    };
                    println!("  {} {}", checkbox, task.text);
                }
                println!();
            }
        }
    }
    Ok(())
}

fn cmd_add(vault: &Vault, note_path: &str, text: &str) -> Result<()> {
    let path = fs::resolve_note(vault, note_path)?;
    let mut content = std::fs::read_to_string(&path)?;

    let task_line = format!("- [ ] {}", text);

    // Append task at end of file
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(&task_line);
    content.push('\n');

    std::fs::write(&path, &content)?;
    println!("{} Added task: {}", "✓".green().bold(), text);
    Ok(())
}

fn cmd_toggle(vault: &Vault, note_path: &str, line: usize, done: bool) -> Result<()> {
    let path = fs::resolve_note(vault, note_path)?;
    let mut lines: Vec<String> = std::fs::read_to_string(&path)?
        .lines()
        .map(|l| l.to_string())
        .collect();

    if line == 0 || line > lines.len() {
        anyhow::bail!("Invalid line number: {}", line);
    }

    let line_idx = line - 1;
    let current_line = lines[line_idx].clone();

    if !current_line.contains("- [ ]")
        && !current_line.contains("- [x]")
        && !current_line.contains("- [X]")
    {
        anyhow::bail!("Line {} is not a task", line);
    }

    let new_line = if done {
        current_line.replace("- [ ]", "- [x]")
    } else {
        current_line
            .replace("- [x]", "- [ ]")
            .replace("- [X]", "- [ ]")
    };

    lines[line_idx] = new_line;
    std::fs::write(&path, lines.join("\n"))?;

    println!(
        "{} Task {} at line {}",
        "✓".green().bold(),
        if done { "completed" } else { "reopened" },
        line
    );
    Ok(())
}

fn cmd_remove(vault: &Vault, note_path: &str, line: usize) -> Result<()> {
    let path = fs::resolve_note(vault, note_path)?;
    let mut lines: Vec<String> = std::fs::read_to_string(&path)?
        .lines()
        .map(|l| l.to_string())
        .collect();

    if line == 0 || line > lines.len() {
        anyhow::bail!("Invalid line number: {}", line);
    }

    let line_idx = line - 1;
    let current_line = lines[line_idx].clone();

    if !current_line.contains("- [ ]")
        && !current_line.contains("- [x]")
        && !current_line.contains("- [X]")
    {
        anyhow::bail!("Line {} is not a task", line);
    }

    let removed = lines.remove(line_idx);
    std::fs::write(&path, lines.join("\n"))?;

    println!("{} Removed task: {}", "✓".green().bold(), removed.trim());
    Ok(())
}
