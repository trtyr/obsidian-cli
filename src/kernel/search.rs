//! Search engine for vault notes.

use anyhow::Result;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use regex::Regex;

use super::index::VaultIndex;
use super::note::Note;

/// Search result.
#[derive(Debug)]
pub struct SearchResult {
    pub note: Note,
    pub score: i64,
    pub matches: Vec<SearchMatch>,
}

/// A matched line in search results.
#[derive(Debug)]
#[allow(dead_code)]
pub struct SearchMatch {
    pub line_num: usize,
    pub line: String,
    pub context_before: Option<String>,
    pub context_after: Option<String>,
}

/// Search options.
#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub regex: bool,
    pub case_sensitive: bool,
    pub context_lines: usize,
    pub path_only: bool,
    pub tag: Option<String>,
    pub property: Option<(String, String)>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            regex: false,
            case_sensitive: false,
            context_lines: 2,
            path_only: false,
            tag: None,
            property: None,
        }
    }
}

/// Search the vault.
pub fn search(index: &VaultIndex, query: &str, opts: &SearchOptions) -> Result<Vec<SearchResult>> {
    let mut results = Vec::new();

    // Build regex pattern
    let pattern = if opts.regex {
        let flags = if opts.case_sensitive { "" } else { "(?i)" };
        Regex::new(&format!("{}{}", flags, query))?
    } else {
        let escaped = regex::escape(query);
        let flags = if opts.case_sensitive { "" } else { "(?i)" };
        Regex::new(&format!("{}{}", flags, escaped))?
    };

    let matcher = SkimMatcherV2::default().ignore_case();

    for note in index.notes.values() {
        // Filter by tag
        if let Some(ref tag) = opts.tag {
            if !note.has_tag(tag) {
                continue;
            }
        }

        // Filter by property
        if let Some((ref key, ref value)) = opts.property {
            match note.property(key) {
                Some(v) if v.contains(value) => {}
                _ => continue,
            }
        }

        // Search in path
        if opts.path_only {
            let path_str = note.path.to_string_lossy();
            if pattern.is_match(&path_str) {
                let score = matcher.fuzzy_match(&path_str, query).unwrap_or(0);
                results.push(SearchResult {
                    note: note.clone(),
                    score,
                    matches: vec![],
                });
            }
            continue;
        }

        // Search in content
        let mut matches = Vec::new();
        let lines: Vec<&str> = note.raw.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            if pattern.is_match(line) {
                let context_before = if opts.context_lines > 0 && i >= opts.context_lines {
                    Some(
                        lines[i - opts.context_lines..i]
                            .join("\n"),
                    )
                } else {
                    None
                };

                let context_after = if opts.context_lines > 0 && i + opts.context_lines < lines.len()
                {
                    Some(
                        lines[i + 1..=i + opts.context_lines]
                            .join("\n"),
                    )
                } else {
                    None
                };

                matches.push(SearchMatch {
                    line_num: i + 1,
                    line: line.to_string(),
                    context_before,
                    context_after,
                });
            }
        }

        // Also search in note name
        if pattern.is_match(&note.name) {
            matches.push(SearchMatch {
                line_num: 0,
                line: note.name.clone(),
                context_before: None,
                context_after: None,
            });
        }

        if !matches.is_empty() {
            let score = matcher.fuzzy_match(&note.name, query).unwrap_or(0)
                + (matches.len() as i64 * 10);
            results.push(SearchResult {
                note: note.clone(),
                score,
                matches,
            });
        }
    }

    // Sort by score
    results.sort_by(|a, b| b.score.cmp(&a.score));
    Ok(results)
}

/// Quick fuzzy search by note name.
#[allow(dead_code)]
pub fn fuzzy_find<'a>(index: &'a VaultIndex, query: &str) -> Vec<(&'a Note, i64)> {
    let matcher = SkimMatcherV2::default().ignore_case();
    let mut results: Vec<_> = index
        .notes
        .values()
        .filter_map(|note| {
            matcher
                .fuzzy_match(&note.name, query)
                .map(|score| (note, score))
        })
        .collect();
    results.sort_by(|a, b| b.1.cmp(&a.1));
    results
}
