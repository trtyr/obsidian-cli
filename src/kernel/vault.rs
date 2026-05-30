//! Vault configuration and discovery.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Obsidian vault configuration.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Vault {
    /// Root path of the vault.
    pub root: PathBuf,
    /// App configuration.
    pub app: AppConfig,
    /// Daily notes configuration.
    pub daily: DailyConfig,
    /// Core plugins state.
    pub core_plugins: CorePlugins,
}

/// `app.json` configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct AppConfig {
    #[serde(default)]
    pub always_update_links: bool,
    #[serde(default = "default_new_file_location")]
    pub new_file_location: String,
    #[serde(default)]
    pub new_file_folder_path: Option<String>,
    #[serde(default)]
    pub attachment_folder_path: Option<String>,
    #[serde(default)]
    pub prompt_delete: bool,
}

fn default_new_file_location() -> String {
    "folder".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            always_update_links: true,
            new_file_location: "folder".to_string(),
            new_file_folder_path: Some("Note".to_string()),
            attachment_folder_path: None,
            prompt_delete: false,
        }
    }
}

/// `daily-notes.json` configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct DailyConfig {
    #[serde(default = "default_daily_folder")]
    pub folder: String,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub template: Option<String>,
}

fn default_daily_folder() -> String {
    "Daily".to_string()
}

impl Default for DailyConfig {
    fn default() -> Self {
        Self {
            folder: "Daily".to_string(),
            format: None,
            template: None,
        }
    }
}

/// Core plugins state.
#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
pub struct CorePlugins {
    #[serde(default)]
    pub daily_notes: bool,
    #[serde(default)]
    pub templates: bool,
    #[serde(default)]
    pub bookmarks: bool,
    #[serde(default)]
    pub properties: bool,
    #[serde(default)]
    pub outline: bool,
    #[serde(default)]
    pub word_count: bool,
}

impl Vault {
    /// Discover and load a vault from a path.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let root = path.as_ref().to_path_buf();
        let obsidian_dir = root.join(".obsidian");

        if !obsidian_dir.exists() {
            anyhow::bail!(
                "Not an Obsidian vault: {} (no .obsidian directory)",
                root.display()
            );
        }

        let app = Self::load_json::<AppConfig>(&obsidian_dir.join("app.json"))
            .unwrap_or_default();
        let daily = Self::load_json::<DailyConfig>(&obsidian_dir.join("daily-notes.json"))
            .unwrap_or_default();
        let core_plugins = Self::load_json::<CorePlugins>(&obsidian_dir.join("core-plugins.json"))
            .unwrap_or_default();

        Ok(Self {
            root,
            app,
            daily,
            core_plugins,
        })
    }

    /// Try to discover vault from current directory or common locations.
    pub fn discover() -> Result<Self> {
        // Try current directory
        let cwd = std::env::current_dir()?;
        if cwd.join(".obsidian").exists() {
            return Self::open(&cwd);
        }

        // Try parent directories
        let mut dir = cwd.parent();
        while let Some(d) = dir {
            if d.join(".obsidian").exists() {
                return Self::open(d);
            }
            dir = d.parent();
        }

        anyhow::bail!(
            "No Obsidian vault found. Run this command inside a vault or use --vault <path>"
        );
    }

    /// Get the daily notes folder path.
    pub fn daily_folder(&self) -> PathBuf {
        self.root.join(&self.daily.folder)
    }

    /// Get the new file folder path.
    #[allow(dead_code)]
    pub fn new_file_folder(&self) -> PathBuf {
        match &self.app.new_file_folder_path {
            Some(folder) => self.root.join(folder),
            None => self.root.clone(),
        }
    }

    /// Get the attachment folder path.
    #[allow(dead_code)]
    pub fn attachment_folder(&self) -> PathBuf {
        match &self.app.attachment_folder_path {
            Some(folder) => self.root.join(folder),
            None => self.root.clone(),
        }
    }

    /// Resolve a note path relative to vault root.
    /// Handles both absolute and relative paths.
    pub fn resolve_path(&self, path: &str) -> PathBuf {
        let p = Path::new(path);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            // Try relative to vault root
            let full = self.root.join(p);
            if full.exists() {
                full
            } else {
                // Try adding .md extension
                let with_ext = self.root.join(format!("{}.md", path));
                if with_ext.exists() {
                    with_ext
                } else {
                    full
                }
            }
        }
    }

    /// Get vault-relative path string.
    pub fn relative_path(&self, path: &Path) -> String {
        path.strip_prefix(&self.root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string()
    }

    /// Get note name from path (without extension).
    pub fn note_name(path: &Path) -> String {
        path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }

    fn load_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))
    }
}
