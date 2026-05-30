//! Note data model and parsing.

use anyhow::Result;
use chrono::{DateTime, Local};
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A parsed Obsidian note.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Note {
    /// File path (absolute).
    pub path: PathBuf,
    /// Note name (filename without extension).
    pub name: String,
    /// Frontmatter properties.
    pub frontmatter: HashMap<String, FrontmatterValue>,
    /// Body content (without frontmatter).
    pub body: String,
    /// Raw content (full file).
    pub raw: String,
    /// Wikilinks found in the note.
    pub wikilinks: Vec<Wikilink>,
    /// Tags found in the note.
    pub tags: Vec<String>,
    /// Tasks found in the note.
    pub tasks: Vec<Task>,
    /// Headings found in the note.
    pub headings: Vec<Heading>,
    /// Word count.
    pub word_count: usize,
    /// Character count.
    pub char_count: usize,
    /// File creation time.
    pub created: Option<DateTime<Local>>,
    /// File modification time.
    pub modified: Option<DateTime<Local>>,
}

/// A frontmatter value.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum FrontmatterValue {
    String(String),
    Number(f64),
    Bool(bool),
    Array(Vec<String>),
    Map(HashMap<String, FrontmatterValue>),
    Null,
}

impl FrontmatterValue {
    #[allow(dead_code)]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn as_array(&self) -> Option<&Vec<String>> {
        match self {
            Self::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn to_display_string(&self) -> String {
        match self {
            Self::String(s) => s.clone(),
            Self::Number(n) => format!("{}", n),
            Self::Bool(b) => format!("{}", b),
            Self::Array(arr) => format!("[{}]", arr.join(", ")),
            Self::Map(_) => "{...}".to_string(),
            Self::Null => "null".to_string(),
        }
    }
}

/// A wikilink reference.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Wikilink {
    /// The target note name/path.
    pub target: String,
    /// Display text (if different from target).
    pub display: Option<String>,
    /// Block reference (if any).
    pub block_ref: Option<String>,
    /// Heading reference (if any).
    pub heading_ref: Option<String>,
    /// Is this an embedded link (![[...]])?
    pub is_embed: bool,
    /// Line number where the link appears.
    pub line: usize,
}

/// A task item.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Task {
    /// Task text.
    pub text: String,
    /// Is it completed?
    pub done: bool,
    /// Line number.
    pub line: usize,
    /// Indentation level.
    pub indent: usize,
    /// Tags within the task.
    pub tags: Vec<String>,
}

/// A heading.
#[derive(Debug, Clone)]
pub struct Heading {
    /// Heading text.
    pub text: String,
    /// Heading level (1-6).
    pub level: u8,
    /// Line number.
    pub line: usize,
}

impl Note {
    /// Parse a note from a file path.
    pub fn parse(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let raw = std::fs::read_to_string(&path)?;
        Self::parse_str(&path, &raw)
    }

    /// Parse a note from string content.
    pub fn parse_str(path: &Path, raw: &str) -> Result<Self> {
        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Parse frontmatter
        let (frontmatter, body) = Self::parse_frontmatter(raw);

        // Extract wikilinks
        let wikilinks = Self::extract_wikilinks(&body);

        // Extract tags
        let mut tags = Self::extract_frontmatter_tags(&frontmatter);
        tags.extend(Self::extract_inline_tags(&body));
        tags.sort();
        tags.dedup();

        // Extract tasks
        let tasks = Self::extract_tasks(&body);

        // Extract headings
        let headings = Self::extract_headings(&body);

        // Word count
        let word_count = Self::count_words(&body);
        let char_count = body.chars().count();

        // File times
        let (created, modified) = Self::get_file_times(&path);

        Ok(Self {
            path: path.to_path_buf(),
            name,
            frontmatter,
            body,
            raw: raw.to_string(),
            wikilinks,
            tags,
            tasks,
            headings,
            word_count,
            char_count,
            created,
            modified,
        })
    }

    fn parse_frontmatter(raw: &str) -> (HashMap<String, FrontmatterValue>, String) {
        let mut frontmatter = HashMap::new();
        let body = if raw.starts_with("---") {
            if let Some(end) = raw[3..].find("\n---") {
                let yaml_str = &raw[3..3 + end];
                let body = raw[3 + end + 4..].trim_start_matches('\n').to_string();

                // Parse YAML frontmatter
                if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(yaml_str) {
                    if let serde_yaml::Value::Mapping(map) = yaml {
                        for (key, value) in map {
                            if let serde_yaml::Value::String(k) = key {
                                frontmatter.insert(k, Self::yaml_to_frontmatter(value));
                            }
                        }
                    }
                }

                body
            } else {
                raw.to_string()
            }
        } else {
            raw.to_string()
        };

        (frontmatter, body)
    }

    fn yaml_to_frontmatter(value: serde_yaml::Value) -> FrontmatterValue {
        match value {
            serde_yaml::Value::String(s) => FrontmatterValue::String(s),
            serde_yaml::Value::Number(n) => {
                FrontmatterValue::Number(n.as_f64().unwrap_or(0.0))
            }
            serde_yaml::Value::Bool(b) => FrontmatterValue::Bool(b),
            serde_yaml::Value::Sequence(seq) => {
                let arr: Vec<String> = seq
                    .iter()
                    .filter_map(|v| match v {
                        serde_yaml::Value::String(s) => Some(s.clone()),
                        _ => Some(format!("{:?}", v)),
                    })
                    .collect();
                FrontmatterValue::Array(arr)
            }
            serde_yaml::Value::Mapping(map) => {
                let mut m = HashMap::new();
                for (k, v) in map {
                    if let serde_yaml::Value::String(key) = k {
                        m.insert(key, Self::yaml_to_frontmatter(v));
                    }
                }
                FrontmatterValue::Map(m)
            }
            serde_yaml::Value::Null => FrontmatterValue::Null,
            _ => FrontmatterValue::String(format!("{:?}", value)),
        }
    }

    fn extract_wikilinks(body: &str) -> Vec<Wikilink> {
        let re = Regex::new(r"(!?)\[\[([^\]]+?)(?:\|([^\]]+?))?\]\]").unwrap();
        let mut links = Vec::new();

        for (line_num, line) in body.lines().enumerate() {
            for cap in re.captures_iter(line) {
                let is_embed = &cap[1] == "!";
                let target_part = &cap[2];
                let display = cap.get(3).map(|m| m.as_str().to_string());

                // Parse target for block/heading refs
                let (target, block_ref, heading_ref) =
                    if let Some(idx) = target_part.find('#') {
                        let t = target_part[..idx].to_string();
                        let ref_part = &target_part[idx + 1..];
                        if ref_part.starts_with('^') {
                            (t, Some(ref_part[1..].to_string()), None)
                        } else {
                            (t, None, Some(ref_part.to_string()))
                        }
                    } else if let Some(idx) = target_part.find('^') {
                        (
                            target_part[..idx].to_string(),
                            Some(target_part[idx + 1..].to_string()),
                            None,
                        )
                    } else {
                        (target_part.to_string(), None, None)
                    };

                links.push(Wikilink {
                    target,
                    display,
                    block_ref,
                    heading_ref,
                    is_embed,
                    line: line_num + 1,
                });
            }
        }

        links
    }

    fn extract_frontmatter_tags(frontmatter: &HashMap<String, FrontmatterValue>) -> Vec<String> {
        let mut tags = Vec::new();
        if let Some(FrontmatterValue::Array(arr)) = frontmatter.get("tags") {
            tags.extend(arr.clone());
        }
        tags
    }

    fn extract_inline_tags(body: &str) -> Vec<String> {
        let re = Regex::new(r"(?:^|\s)#([a-zA-Z\u4e00-\u9fff][a-zA-Z0-9\u4e00-\u9fff_/\-]*)").unwrap();
        re.captures_iter(body)
            .map(|cap| cap[1].to_string())
            .collect()
    }

    fn extract_tasks(body: &str) -> Vec<Task> {
        let re = Regex::new(r"^(\s*)- \[([ xX])\] (.+)$").unwrap();
        let tag_re = Regex::new(r"#([a-zA-Z\u4e00-\u9fff][a-zA-Z0-9\u4e00-\u9fff_/\-]*)").unwrap();

        body.lines()
            .enumerate()
            .filter_map(|(line_num, line)| {
                re.captures(line).map(|cap| {
                    let indent = cap[1].len();
                    let done = &cap[2] != " ";
                    let text = cap[3].to_string();
                    let tags: Vec<String> = tag_re
                        .captures_iter(&text)
                        .map(|c| c[1].to_string())
                        .collect();

                    Task {
                        text,
                        done,
                        line: line_num + 1,
                        indent,
                        tags,
                    }
                })
            })
            .collect()
    }

    fn extract_headings(body: &str) -> Vec<Heading> {
        let re = Regex::new(r"^(#{1,6})\s+(.+)$").unwrap();
        body.lines()
            .enumerate()
            .filter_map(|(line_num, line)| {
                re.captures(line).map(|cap| Heading {
                    text: cap[2].to_string(),
                    level: cap[1].len() as u8,
                    line: line_num + 1,
                })
            })
            .collect()
    }

    fn count_words(body: &str) -> usize {
        // Count CJK characters individually + English words
        let cjk_re = Regex::new(r"[\u4e00-\u9fff\u3400-\u4dbf]").unwrap();
        let word_re = Regex::new(r"[a-zA-Z]+").unwrap();
        let cjk_count = cjk_re.find_iter(body).count();
        let word_count = word_re.find_iter(body).count();
        cjk_count + word_count
    }

    fn get_file_times(path: &Path) -> (Option<DateTime<Local>>, Option<DateTime<Local>>) {
        let meta = std::fs::metadata(path).ok();
        let created = meta
            .as_ref()
            .and_then(|m| m.created().ok())
            .map(|t| DateTime::from(t));
        let modified = meta
            .as_ref()
            .and_then(|m| m.modified().ok())
            .map(|t| DateTime::from(t));
        (created, modified)
    }

    /// Get a property value as string.
    pub fn property(&self, key: &str) -> Option<String> {
        self.frontmatter.get(key).map(|v| v.to_display_string())
    }

    /// Get tags property as vector.
    pub fn property_tags(&self) -> Vec<String> {
        match self.frontmatter.get("tags") {
            Some(FrontmatterValue::Array(arr)) => arr.clone(),
            Some(FrontmatterValue::String(s)) => vec![s.clone()],
            _ => Vec::new(),
        }
    }

    /// Get the note's aliases.
    pub fn aliases(&self) -> Vec<String> {
        match self.frontmatter.get("aliases") {
            Some(FrontmatterValue::Array(arr)) => arr.clone(),
            Some(FrontmatterValue::String(s)) => vec![s.clone()],
            _ => Vec::new(),
        }
    }

    /// Check if note has a specific tag.
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    /// Get all tasks.
    pub fn get_tasks(&self) -> &[Task] {
        &self.tasks
    }

    /// Get incomplete tasks.
    #[allow(dead_code)]
    pub fn pending_tasks(&self) -> Vec<&Task> {
        self.tasks.iter().filter(|t| !t.done).collect()
    }
}
