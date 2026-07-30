use std::path::Path;

use crate::config::schema::{default_cli_tools, AppConfig};

/// Append CLI tools introduced after the user's config was persisted, so new
/// entries (e.g. "kimi") show up without resetting existing settings.
fn merge_new_cli_tools(config: &mut AppConfig) {
    for tool in default_cli_tools() {
        if !config.hook.cli_tools.iter().any(|t| t.id == tool.id) {
            config.hook.cli_tools.push(tool);
        }
    }
}

pub fn load_or_create(config_dir: &Path) -> AppConfig {
    let config_path = config_dir.join("config.toml");
    if config_path.exists() {
        match std::fs::read_to_string(&config_path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(mut config) => {
                    tracing::info!("Loaded config from {}", config_path.display());
                    merge_new_cli_tools(&mut config);
                    return config;
                }
                Err(e) => {
                    tracing::warn!("Failed to parse config: {}, using defaults", e);
                }
            },
            Err(e) => {
                tracing::warn!("Failed to read config: {}, using defaults", e);
            }
        }
    }
    let config = AppConfig::default();
    save(&config_dir, &config);
    config
}

pub fn save(config_dir: &Path, config: &AppConfig) {
    let config_path = config_dir.join("config.toml");
    match toml::to_string_pretty(config) {
        Ok(content) => {
            if let Err(e) = std::fs::write(&config_path, content) {
                tracing::error!("Failed to save config: {}", e);
            }
        }
        Err(e) => {
            tracing::error!("Failed to serialize config: {}", e);
        }
    }
}
