use crate::config::persistence;
use crate::config::schema::AppConfig;
use crate::state::AppState;
use std::net::UdpSocket;
use tauri::{AppHandle, State};
#[cfg(desktop)]
use tauri_plugin_autostart::ManagerExt;

/// Apply the run-on-startup setting to the OS auto-launch registration.
#[cfg(desktop)]
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
    mut config: AppConfig,
) -> Result<(), String> {
    // Detect language change before saving
    #[cfg(desktop)]
    let old_language = {
        let current = state.config.read().await;
        current.general.language.clone()
    };
    #[cfg(desktop)]
    let language_changed = old_language != config.general.language;
    #[cfg(not(desktop))]
    let _ = &app;

    // The alert-server connection section is backend-owned: device
    // credentials are issued by registration and the push token arrives
    // asynchronously from the native bridge. A settings save must never
    // overwrite them with a stale frontend snapshot (only `auto_connect`
    // is edited through this command). Merge, persist and swap under a
    // single write lock so a concurrent registration can't interleave.
    let run_on_startup = config.general.run_on_startup;
    {
        let mut current = state.config.write().await;
        let auto_connect = config.server.auto_connect;
        config.server = current.server.clone();
        config.server.auto_connect = auto_connect;
        persistence::save(&state.config_dir, &config);
        *current = config;
    }
    #[cfg(desktop)]
    apply_autostart(&app, run_on_startup);

    // Rebuild tray menu if language changed
    #[cfg(desktop)]
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
    #[cfg(not(desktop))]
    let _ = &app;
    let mut config = persistence::embedded_default();
    // Keep the alert-server connection: resetting preferences must not
    // silently unregister the device or drop its credentials.
    {
        let current = state.config.read().await;
        config.server = current.server.clone();
    }
    persistence::save(&state.config_dir, &config);
    #[cfg(desktop)]
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
