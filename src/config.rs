use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub genius: GeniusConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Deserialize)]
pub struct GeniusConfig {
    pub access_token: String,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path))?;
        
        let config: Config = toml::from_str(&contents)
            .with_context(|| "Failed to parse config file")?;
        
        Ok(config)
    }
}
