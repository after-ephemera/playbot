use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

/// Application configuration loaded from a TOML file.
#[derive(Debug, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
}

/// Database configuration section.
#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    /// Path to the SQLite database file. Supports `~` for the home directory.
    pub path: String,
}

impl Config {
    /// Get the default application directory (`~/.pb/`).
    pub fn get_app_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("Failed to get HOME environment variable")?;
        Ok(PathBuf::from(home).join(".pb"))
    }

    /// Get the default config file path (`~/.pb/config.toml`).
    pub fn get_default_config_path() -> Result<PathBuf> {
        Ok(Self::get_app_dir()?.join("config.toml"))
    }

    /// Get the default database path (`~/.pb/playbot.db`).
    #[allow(dead_code)]
    pub fn get_default_db_path() -> Result<PathBuf> {
        Ok(Self::get_app_dir()?.join("playbot.db"))
    }

    /// Ensure the application directory (`~/.pb/`) exists, creating it if needed.
    pub fn ensure_app_dir() -> Result<PathBuf> {
        let app_dir = Self::get_app_dir()?;
        if !app_dir.exists() {
            fs::create_dir_all(&app_dir)
                .with_context(|| format!("Failed to create directory: {:?}", app_dir))?;
            println!("✨ Created playbot directory at {:?}", app_dir);
        }
        Ok(app_dir)
    }

    /// Load configuration from a TOML file at the given path.
    ///
    /// Expands `~` to the home directory in the database path.
    pub fn load(path: &str) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path))?;

        let mut config: Config =
            toml::from_str(&contents).with_context(|| "Failed to parse config file")?;

        // Expand ~ in database path if present
        if config.database.path.starts_with("~/") {
            let home = std::env::var("HOME").context("Failed to get HOME environment variable")?;
            config.database.path = config.database.path.replacen("~", &home, 1);
        }

        Ok(config)
    }
}
