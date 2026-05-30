//! Configuration management for obsidian-cli.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Default vault path.
    pub default_vault: Option<String>,
    /// Named vaults (name → path).
    #[serde(default)]
    pub vaults: std::collections::HashMap<String, String>,
}

impl Config {
    /// Get the config file path.
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?
            .join("obsidian-cli");
        Ok(config_dir.join("config.json"))
    }

    /// Load config from file.
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&path)
            .context("Could not read config file")?;
        let config: Self = serde_json::from_str(&content)
            .context("Could not parse config file")?;
        Ok(config)
    }

    /// Save config to file.
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .context("Could not create config directory")?;
        }

        let content = serde_json::to_string_pretty(self)
            .context("Could not serialize config")?;
        std::fs::write(&path, content)
            .context("Could not write config file")?;
        Ok(())
    }

    /// Get the default vault path.
    pub fn get_vault(&self) -> Option<&str> {
        self.default_vault.as_deref()
    }

    /// Set the default vault path.
    pub fn set_vault(&mut self, path: String) {
        self.default_vault = Some(path);
    }

    /// Remove the default vault path.
    pub fn unset_vault(&mut self) {
        self.default_vault = None;
    }

    /// Add a named vault.
    #[allow(dead_code)]
    pub fn add_vault(&mut self, name: String, path: String) {
        self.vaults.insert(name, path);
    }

    /// Remove a named vault.
    #[allow(dead_code)]
    pub fn remove_vault(&mut self, name: &str) -> Option<String> {
        self.vaults.remove(name)
    }

    /// Get a named vault path.
    #[allow(dead_code)]
    pub fn get_named_vault(&self, name: &str) -> Option<&str> {
        self.vaults.get(name).map(|s| s.as_str())
    }
}
