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

    /// Authentication token (JWT) - loaded from credentials
    #[serde(skip)]
    pub token: Option<String>,

    /// API key - loaded from credentials
    #[serde(skip)]
    pub api_key: Option<String>,

    /// Username - loaded from credentials
    #[serde(skip)]
    pub username: Option<String>,

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

/// Credentials stored in ~/.flaglite/credentials.json
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Credentials {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
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

    /// Get the credentials directory path (~/.flaglite)
    pub fn credentials_dir() -> Result<PathBuf> {
        let dir = dirs::home_dir()
            .context("Could not determine home directory")?
            .join(".flaglite");
        Ok(dir)
    }

    /// Get the credentials file path
    pub fn credentials_path() -> Result<PathBuf> {
        Ok(Self::credentials_dir()?.join("credentials.json"))
    }

    /// Load config from disk, or return defaults
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        let mut config = if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read config from {}", path.display()))?;
            toml::from_str(&content)
                .with_context(|| format!("Failed to parse config from {}", path.display()))?
        } else {
            Self::default()
        };

        // Load credentials
        config.load_credentials()?;

        // Apply env var overrides for API key
        if let Ok(key) = std::env::var("FLAGLITE_API_KEY") {
            if !key.is_empty() {
                config.api_key = Some(key);
            }
        }

        Ok(config)
    }

    /// Load credentials from ~/.flaglite/credentials.json
    fn load_credentials(&mut self) -> Result<()> {
        let path = Self::credentials_path()?;

        if !path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read credentials from {}", path.display()))?;

        let creds: Credentials = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse credentials from {}", path.display()))?;

        self.token = creds.token;
        self.api_key = creds.api_key;
        self.username = creds.username;

        // Use api_url from credentials if set
        if let Some(url) = creds.api_url {
            self.api_url = url;
        }

        Ok(())
    }

    /// Save config to disk (not credentials)
    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir()?;
        let path = Self::config_path()?;

        // Create directory if needed
        if !dir.exists() {
            fs::create_dir_all(&dir)
                .with_context(|| format!("Failed to create config directory: {}", dir.display()))?;
        }

        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;

        fs::write(&path, content)
            .with_context(|| format!("Failed to write config to {}", path.display()))?;

        Ok(())
    }

    /// Save credentials to ~/.flaglite/credentials.json
    pub fn save_credentials(&self) -> Result<()> {
        let dir = Self::credentials_dir()?;
        let path = Self::credentials_path()?;

        // Create directory if needed
        if !dir.exists() {
            fs::create_dir_all(&dir)
                .with_context(|| format!("Failed to create credentials directory: {}", dir.display()))?;
        }

        let creds = Credentials {
            api_url: Some(self.api_url.clone()),
            api_key: self.api_key.clone(),
            username: self.username.clone(),
            token: self.token.clone(),
        };

        let content =
            serde_json::to_string_pretty(&creds).context("Failed to serialize credentials")?;

        fs::write(&path, &content)
            .with_context(|| format!("Failed to write credentials to {}", path.display()))?;

        // Set restrictive permissions (0600)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&path, perms)?;
        }

        Ok(())
    }

    /// Delete credentials file
    pub fn delete_credentials() -> Result<()> {
        let path = Self::credentials_path()?;

        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("Failed to delete credentials at {}", path.display()))?;
        }

        Ok(())
    }

    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.token.is_some() || self.api_key.is_some()
    }

    /// Get the API token, or error if not authenticated
    pub fn require_token(&self) -> Result<&str> {
        // Prefer API key
        if let Some(key) = &self.api_key {
            return Ok(key);
        }
        self.token
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("Not logged in. Run `flaglite signup` or `flaglite login`"))
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

    /// Clear authentication (for logout)
    pub fn clear_auth(&mut self) {
        self.token = None;
        self.api_key = None;
        self.username = None;
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_url: default_api_url(),
            token: None,
            api_key: None,
            username: None,
            project_id: None,
            environment: None,
        }
    }
}
