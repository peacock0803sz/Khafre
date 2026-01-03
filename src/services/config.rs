//! Configuration management service

use std::path::PathBuf;

use anyhow::Result;

use crate::types::config::Config;

/// Get the configuration directory path
pub fn get_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("khafre"))
}

/// Get the configuration file path
pub fn get_config_path() -> Option<PathBuf> {
    get_config_dir().map(|p| p.join("config.toml"))
}

/// Load configuration from file
pub fn load_config() -> Result<Config> {
    let config_path = get_config_path().ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

    if !config_path.exists() {
        log::info!("Config file not found, using defaults");
        return Ok(Config::default());
    }

    let content = std::fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&content)?;

    log::info!("Loaded config from {:?}", config_path);
    Ok(config)
}

/// Load project-specific configuration
pub fn load_project_config(project_path: &std::path::Path) -> Result<Option<Config>> {
    let config_path = project_path.join(".khafre.toml");

    if !config_path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&content)?;

    log::info!("Loaded project config from {:?}", config_path);
    Ok(Some(config))
}
