use crate::config::persistence;
use crate::config::schema::AppConfig;
use crate::state::AppState;
use std::net::UdpSocket;
use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;

/// Apply the run-on-startup setting to the OS auto-launch registration.
fn apply_autostart(app: &AppHandle, enabled: bool) {
    let autostart = app.autolaunch();
    if autostart.is_enabled().unwrap_or(false) == enabled {
        return;
    }
    let r = if enabled {
        autostart.enable()
    } else {
        autostart.disable()
    };
    if let Err(e) = r {
        tracing::warn!("Failed to update autostart registration: {}", e);
    }
}

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state.config.read().await;
    Ok(config.clone())
}

#[tauri::command]
pub async fn save_config(
    app: AppHandle,
    state: State<'_, AppState>,
    config: AppConfig,
) -> Result<(), String> {
    // Detect language change before saving
    let old_language = {
        let current = state.config.read().await;
        current.general.language.clone()
    };
    let language_changed = old_language != config.general.language;

    persistence::save(&state.config_dir, &config);
    apply_autostart(&app, config.general.run_on_startup);
    let mut current = state.config.write().await;
    *current = config;
    drop(current);

    // Rebuild tray menu if language changed
    if language_changed {
        let new_language = state.config.read().await.general.language.clone();
        if let Err(e) = crate::tray::menu::rebuild_tray(&app, &new_language) {
            tracing::warn!("Failed to rebuild tray menu: {}", e);
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn reset_config(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<AppConfig, String> {
    let config = AppConfig::default();
    persistence::save(&state.config_dir, &config);
    apply_autostart(&app, config.general.run_on_startup);
    let mut current = state.config.write().await;
    *current = config.clone();
    Ok(config)
}

#[tauri::command]
pub fn get_host_ip() -> Result<String, String> {
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
    socket
        .connect("8.8.8.8:80")
        .map_err(|e| e.to_string())?;
    let addr = socket
        .local_addr()
        .map_err(|e| e.to_string())?;
    Ok(addr.ip().to_string())
}

#[tauri::command]
pub async fn get_wan_ip() -> Result<String, String> {
    let resp = reqwest::get("https://api.ipify.org")
        .await
        .map_err(|e| e.to_string())?;
    let text = resp.text().await.map_err(|e| e.to_string())?;
    Ok(text.trim().to_string())
}
