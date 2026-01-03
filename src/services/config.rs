//! Configuration management service

use std::path::PathBuf;

use anyhow::Result;

use crate::types::config::{Config, DevConfig};

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
#[allow(dead_code)]
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

/// Load development configuration overrides
///
/// Looks for .khafre.dev.json in the project directory.
/// This file is meant for local development overrides and should be gitignored.
pub fn load_dev_config(project_path: &std::path::Path) -> Result<Option<DevConfig>> {
    let config_path = project_path.join(".khafre.dev.json");

    if !config_path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&config_path)?;
    let config: DevConfig = serde_json::from_str(&content)?;

    log::info!("Loaded dev config from {:?}", config_path);
    Ok(Some(config))
}

/// Load full configuration with all overrides applied
///
/// Order of precedence (later overrides earlier):
/// 1. Global config (~/.config/khafre/config.toml)
/// 2. Project config (.khafre.toml)
/// 3. Dev config (.khafre.dev.json)
pub fn load_full_config(project_path: Option<&std::path::Path>) -> Result<Config> {
    // Start with global config
    let mut config = load_config()?;

    if let Some(project_path) = project_path {
        // Apply project config if present
        if let Ok(Some(project_config)) = load_project_config(project_path) {
            // Merge project config (project takes precedence)
            config = project_config;
        }

        // Apply dev config overrides if present
        if let Ok(Some(dev_config)) = load_dev_config(project_path) {
            config = dev_config.apply_to(config);
        }
    }

    Ok(config)
}
