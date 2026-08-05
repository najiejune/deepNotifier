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
                    // Keep the user's file for inspection instead of silently
                    // destroying it: an unparseable config is backed up next to
                    // the original before defaults are written.
                    let backup = config_dir.join("config.toml.bak");
                    if let Err(be) = std::fs::copy(&config_path, &backup) {
                        tracing::warn!("Failed to back up unparseable config: {}", be);
                    }
                    tracing::warn!(
                        "Failed to parse config: {}, backed up to {} and using defaults",
                        e,
                        backup.display()
                    );
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression: configs written before newer sections/fields existed must
    /// still load — missing parts fall back to defaults instead of resetting
    /// the whole file (which used to silently wipe the language setting).
    #[test]
    fn old_config_missing_newer_fields_still_loads() {
        let toml = r#"
[general]
language = "zh"
mode = "push"
run_on_startup = true
minimize_to_tray = true
close_to_tray = true

[notification]
sound_enabled = false
sound_file = "bell"
sound_volume = 0.5
marquee_enabled = true
tray_enabled = true
max_history = 100
"#;
        let config: AppConfig = toml::from_str(toml).expect("old config must parse");
        assert_eq!(config.general.language, "zh");
        assert!(!config.notification.sound_enabled);
        assert_eq!(config.notification.max_history, 100);
        // Missing sections/fields fall back to defaults.
        assert_eq!(config.hook.approval_timeout_secs, 120);
        assert_eq!(config.todo.push_port, 3928);
        assert_eq!(config.marquee.tracks, 2);
    }
}

