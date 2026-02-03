use anyhow::{Context, Result};

pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:flaglite.db?mode=rwc".to_string());

        let jwt_secret = std::env::var("JWT_SECRET")
            .context("JWT_SECRET environment variable is required")?;

        Ok(Config {
            database_url,
            jwt_secret,
        })
    }
}
