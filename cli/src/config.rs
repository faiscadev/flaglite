//! Configuration management for FlagLite CLI

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const DEFAULT_API_URL: &str = "https://api.flaglite.dev";

/// CLI configuration stored in ~/.config/flaglite/config.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// API base URL
    #[serde(default = "default_api_url")]
    pub api_url: String,

    /// Authentication token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,

    /// Default project ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,

    /// Default environment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
}

fn default_api_url() -> String {
    DEFAULT_API_URL.to_string()
}

impl Config {
    /// Get the config directory path
    pub fn config_dir() -> Result<PathBuf> {
        let dir = dirs::config_dir()
            .context("Could not determine config directory")?
            .join("flaglite");
        Ok(dir)
    }

    /// Get the config file path
    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    /// Load config from disk, or return defaults
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config from {}", path.display()))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config from {}", path.display()))?;

        Ok(config)
    }

    /// Save config to disk
    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir()?;
        let path = Self::config_path()?;

        // Create directory if needed
        if !dir.exists() {
            fs::create_dir_all(&dir)
                .with_context(|| format!("Failed to create config directory: {}", dir.display()))?;
        }

        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        fs::write(&path, content)
            .with_context(|| format!("Failed to write config to {}", path.display()))?;

        // Set restrictive permissions on the config file (contains token)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&path, perms)?;
        }

        Ok(())
    }

    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    /// Get the API token, or error if not authenticated
    pub fn require_token(&self) -> Result<&str> {
        self.token
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("Not authenticated. Run 'flaglite login' first."))
    }

    /// Get the project ID, or error if not set
    pub fn require_project(&self) -> Result<&str> {
        self.project_id
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("No project selected. Run 'flaglite projects use <id>' first."))
    }

    /// Get the environment, defaulting to "development"
    pub fn get_environment(&self) -> &str {
        self.environment.as_deref().unwrap_or("development")
    }

    /// Clear authentication
    pub fn clear_auth(&mut self) {
        self.token = None;
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_url: default_api_url(),
            token: None,
            project_id: None,
            environment: None,
        }
    }
}
