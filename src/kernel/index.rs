//! Vault indexer — scans and caches note metadata.

use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use walkdir::WalkDir;

use super::note::Note;
use super::vault::Vault;

/// Vault index for fast lookups.
#[derive(Debug)]
pub struct VaultIndex {
    /// All notes by path.
    pub notes: HashMap<PathBuf, Note>,
    /// Notes by name (lowercase) → paths.
    pub by_name: HashMap<String, Vec<PathBuf>>,
    /// All tags → note paths.
    pub tag_index: HashMap<String, Vec<PathBuf>>,
    /// Backlinks: target_name → source_paths.
    pub backlinks: HashMap<String, Vec<PathBuf>>,
    /// Unresolved links: (source_path, target_name).
    pub unresolved: Vec<(PathBuf, String)>,
}

impl VaultIndex {
    /// Build index for the entire vault.
    pub fn build(vault: &Vault) -> Result<Self> {
        let mut notes = HashMap::new();
        let mut by_name: HashMap<String, Vec<PathBuf>> = HashMap::new();
        let mut tag_index: HashMap<String, Vec<PathBuf>> = HashMap::new();

        // Scan all markdown files
        for entry in WalkDir::new(&vault.root)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| !is_hidden(e))
        {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            if path.extension().map_or(true, |e| e != "md") {
                continue;
            }

            // Skip .obsidian directory
            if path.starts_with(vault.root.join(".obsidian")) {
                continue;
            }

            match Note::parse(path) {
                Ok(note) => {
                    let name_lower = note.name.to_lowercase();

                    // Index by name
                    by_name
                        .entry(name_lower)
                        .or_default()
                        .push(path.to_path_buf());

                    // Index tags
                    for tag in &note.tags {
                        tag_index
                            .entry(tag.clone())
                            .or_default()
                            .push(path.to_path_buf());
                    }

                    notes.insert(path.to_path_buf(), note);
                }
                Err(_) => continue,
            }
        }

        // Build backlinks and unresolved links
        let mut backlinks: HashMap<String, Vec<PathBuf>> = HashMap::new();
        let mut unresolved = Vec::new();

        for (path, note) in &notes {
            for link in &note.wikilinks {
                if link.is_embed {
                    continue; // Skip embeds for backlinks
                }

                let target_name = link.target.to_lowercase();
                if by_name.contains_key(&target_name) {
                    backlinks
                        .entry(link.target.clone())
                        .or_default()
                        .push(path.clone());
                } else {
                    unresolved.push((path.clone(), link.target.clone()));
                }
            }
        }

        Ok(Self {
            notes,
            by_name,
            tag_index,
            backlinks,
            unresolved,
        })
    }

    /// Find a note by name (case-insensitive).
    pub fn find_by_name(&self, name: &str) -> Option<&Note> {
        let name_lower = name.to_lowercase();
        self.by_name
            .get(&name_lower)
            .and_then(|paths| paths.first())
            .and_then(|path| self.notes.get(path))
    }

    /// Find notes by tag.
    pub fn find_by_tag(&self, tag: &str) -> Vec<&Note> {
        self.tag_index
            .get(tag)
            .map(|paths| {
                paths
                    .iter()
                    .filter_map(|p| self.notes.get(p))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get backlinks for a note name.
    pub fn get_backlinks(&self, name: &str) -> Vec<&Note> {
        self.backlinks
            .get(name)
            .map(|paths| {
                paths
                    .iter()
                    .filter_map(|p| self.notes.get(p))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all unique tags.
    pub fn all_tags(&self) -> Vec<(&String, usize)> {
        let mut tags: Vec<_> = self
            .tag_index
            .iter()
            .map(|(tag, paths)| (tag, paths.len()))
            .collect();
        tags.sort_by(|a, b| b.1.cmp(&a.1));
        tags
    }

    /// Get orphan notes (no incoming or outgoing links).
    pub fn orphans(&self) -> Vec<&Note> {
        let linked: std::collections::HashSet<&PathBuf> = self
            .backlinks
            .values()
            .flatten()
            .chain(self.notes.values().flat_map(|n| {
                n.wikilinks.iter().filter_map(|l| {
                    self.by_name
                        .get(&l.target.to_lowercase())
                        .and_then(|ps| ps.first())
                })
            }))
            .collect();

        self.notes
            .iter()
            .filter(|(path, note)| {
                !linked.contains(path) && note.wikilinks.is_empty()
            })
            .map(|(_, note)| note)
            .collect()
    }

    /// Get dead-end notes (have outgoing links but no incoming).
    pub fn deadends(&self) -> Vec<&Note> {
        let has_backlink: std::collections::HashSet<&PathBuf> =
            self.backlinks.values().flatten().collect();

        self.notes
            .iter()
            .filter(|(path, note)| {
                !note.wikilinks.is_empty() && !has_backlink.contains(path)
            })
            .map(|(_, note)| note)
            .collect()
    }
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.') && s != ".obsidian")
        .unwrap_or(false)
}
