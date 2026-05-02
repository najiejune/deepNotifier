use crate::config::schema::MarqueeConfig;
use crate::notifier::marquee::reposition_marquee;
use crate::state::AppState;
use tauri::{Emitter, Manager, State};

fn marquee_window_labels(app: &tauri::AppHandle) -> Vec<String> {
    app.webview_windows()
        .keys()
        .filter(|label| *label == "marquee" || label.starts_with("marquee-"))
        .cloned()
        .collect()
}

fn emit_to_all_marquee_windows(
    app: &tauri::AppHandle,
    event: &str,
    payload: &(impl Clone + serde::Serialize),
) {
    for label in marquee_window_labels(app) {
        if let Some(window) = app.get_webview_window(&label) {
            let _ = window.emit(event, payload);
        }
    }
}

fn show_all_marquees(app: &tauri::AppHandle, cfg: &MarqueeConfig) {
    for label in marquee_window_labels(app) {
        if let Some(window) = app.get_webview_window(&label) {
            reposition_marquee(&window, cfg);
            let _ = window.show();
            let _ = window.set_ignore_cursor_events(true);
        }
    }
}

fn hide_all_marquees(app: &tauri::AppHandle) {
    for label in marquee_window_labels(app) {
        if let Some(window) = app.get_webview_window(&label) {
            let _ = window.hide();
        }
    }
}

#[tauri::command]
pub async fn show_marquee(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    text: String,
) -> Result<(), String> {
    let marquee_cfg: MarqueeConfig = {
        let config = state.config.read().await;
        config.marquee.clone()
    };

    emit_to_all_marquee_windows(&app, "marquee-text", &text);
    emit_to_all_marquee_windows(&app, "marquee-config", &marquee_cfg);
    show_all_marquees(&app, &marquee_cfg);

    let duration = marquee_cfg.duration_secs;
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(duration as u64)).await;
        hide_all_marquees(&handle);
    });

    Ok(())
}

#[tauri::command]
pub async fn hide_marquee(app: tauri::AppHandle) -> Result<(), String> {
    hide_all_marquees(&app);
    Ok(())
}
