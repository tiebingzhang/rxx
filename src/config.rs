use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub user_id: String,
    pub server_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;
        let config: Config = toml::from_str(&content).context("Failed to parse config file")?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;
        fs::write(&path, content)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;
        println!("Config saved to {:?}", path);
        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        Ok(PathBuf::from(home).join(".rxx.conf"))
    }
}
