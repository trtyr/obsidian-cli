//! Output formatting utilities.

use colored::Colorize;

use super::note::{Note, Task, Heading, Wikilink};
use super::search::SearchResult;

/// Format a note for display.
#[allow(dead_code)]
pub fn format_note(note: &Note, verbose: bool) -> String {
    let mut out = String::new();

    // Header
    out.push_str(&format!(
        "{} {}\n",
        "▸".cyan().bold(),
        note.name.bold().white()
    ));

    if verbose {
        out.push_str(&format!(
            "  {} {}\n",
            "Path:".dimmed(),
            note.path.display().to_string().dimmed()
        ));

        if let Some(created) = note.created {
            out.push_str(&format!(
                "  {} {}\n",
                "Created:".dimmed(),
                created.format("%Y-%m-%d %H:%M").to_string().dimmed()
            ));
        }

        if let Some(modified) = note.modified {
            out.push_str(&format!(
                "  {} {}\n",
                "Modified:".dimmed(),
                modified.format("%Y-%m-%d %H:%M").to_string().dimmed()
            ));
        }

        out.push_str(&format!(
            "  {} {} words, {} chars\n",
            "Stats:".dimmed(),
            note.word_count,
            note.char_count
        ));

        if !note.tags.is_empty() {
            out.push_str(&format!(
                "  {} {}\n",
                "Tags:".dimmed(),
                note.tags
                    .iter()
                    .map(|t| format!("#{}", t))
                    .collect::<Vec<_>>()
                    .join(" ")
                    .dimmed()
            ));
        }

        if !note.wikilinks.is_empty() {
            out.push_str(&format!(
                "  {} {} links\n",
                "Links:".dimmed(),
                note.wikilinks.len()
            ));
        }
    }

    out
}

/// Format search results.
pub fn format_search_results(results: &[SearchResult], max_results: usize) -> String {
    let mut out = String::new();

    if results.is_empty() {
        return "No results found.".dimmed().to_string();
    }

    out.push_str(&format!(
        "{} {} result(s)\n\n",
        "Found".green().bold(),
        results.len()
    ));

    for (i, result) in results.iter().take(max_results).enumerate() {
        out.push_str(&format!(
            "{}. {} {}\n",
            (i + 1).to_string().cyan(),
            "▸".dimmed(),
            result.note.name.bold()
        ));

        for m in &result.matches {
            if m.line_num > 0 {
                out.push_str(&format!(
                    "   {} {}\n",
                    format!("L{}", m.line_num).dimmed(),
                    highlight_match(&m.line, &result.note.name)
                ));
            }
        }
        out.push('\n');
    }

    out
}

fn highlight_match(line: &str, query: &str) -> String {
    // Simple highlight — wrap matches in ** **
    let lower = line.to_lowercase();
    let query_lower = query.to_lowercase();
    if let Some(pos) = lower.find(&query_lower) {
        let before = &line[..pos];
        let matched = &line[pos..pos + query.len()];
        let after = &line[pos + query.len().min(line.len() - pos)..];
        format!("{}{}{}", before.dimmed(), matched.red().bold(), after.dimmed())
    } else {
        line.dimmed().to_string()
    }
}

/// Format a task list.
pub fn format_tasks(tasks: &[&Task]) -> String {
    let mut out = String::new();

    if tasks.is_empty() {
        return "No tasks found.".dimmed().to_string();
    }

    let done = tasks.iter().filter(|t| t.done).count();
    let pending = tasks.len() - done;

    out.push_str(&format!(
        "{} {} total, {} {}, {} {}\n\n",
        "Tasks:".bold(),
        tasks.len(),
        pending.to_string().yellow(),
        "pending",
        done.to_string().green(),
        "done"
    ));

    for task in tasks {
        let checkbox = if task.done {
            "[x]".green()
        } else {
            "[ ]".yellow()
        };
        let indent = "  ".repeat(task.indent / 2);
        out.push_str(&format!("{}{} {}\n", indent, checkbox, task.text));
    }

    out
}

/// Format headings as outline.
pub fn format_outline(headings: &[Heading]) -> String {
    let mut out = String::new();

    if headings.is_empty() {
        return "No headings found.".dimmed().to_string();
    }

    for heading in headings {
        let indent = "  ".repeat((heading.level - 1) as usize);
        let prefix = "#".repeat(heading.level as usize);
        out.push_str(&format!(
            "{}{} {} {}\n",
            indent,
            prefix.dimmed(),
            heading.text,
            format!("(L{})", heading.line).dimmed()
        ));
    }

    out
}

/// Format wikilinks.
pub fn format_links(links: &[Wikilink]) -> String {
    let mut out = String::new();

    if links.is_empty() {
        return "No links found.".dimmed().to_string();
    }

    for link in links {
        let prefix = if link.is_embed { "!" } else { "" };
        let target = if let Some(ref h) = link.heading_ref {
            format!("{}#{}", link.target, h)
        } else if let Some(ref b) = link.block_ref {
            format!("{}^{}", link.target, b)
        } else {
            link.target.clone()
        };

        let display = link
            .display
            .as_deref()
            .unwrap_or(&target);

        out.push_str(&format!(
            "  {} {}{}\n",
            "→".cyan(),
            prefix,
            if display != target {
                format!("{}|{}", target, display)
            } else {
                target
            }
        ));
    }

    out
}

/// Format word count stats.
pub fn format_wordcount(note: &Note) -> String {
    format!(
        "{}\n  {} {}\n  {} {}\n  {} {}\n  {} {}",
        "Word Count:".bold(),
        "Words:".dimmed(),
        note.word_count,
        "Characters:".dimmed(),
        note.char_count,
        "Lines:".dimmed(),
        note.raw.lines().count(),
        "Tasks:".dimmed(),
        note.tasks.len()
    )
}
