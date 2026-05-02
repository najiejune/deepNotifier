use std::path::Path;

use crate::config::schema::AppConfig;

pub fn load_or_create(config_dir: &Path) -> AppConfig {
    let config_path = config_dir.join("config.toml");
    if config_path.exists() {
        match std::fs::read_to_string(&config_path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(config) => {
                    tracing::info!("Loaded config from {}", config_path.display());
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
