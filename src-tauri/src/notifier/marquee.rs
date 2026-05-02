use crate::config::schema::{MarqueeConfig, MarqueePosition};
use tauri::{Emitter, Manager, PhysicalPosition};

pub(crate) fn reposition_marquee(window: &tauri::WebviewWindow, cfg: &MarqueeConfig) {
    let h = cfg.height;
    if let Ok(Some(monitor)) = window.current_monitor() {
        let screen = monitor.size();
        let scale = monitor.scale_factor();
        let logical_h = (screen.height as f64 / scale) as u32;
        let x = monitor.position().x;
        let y = match cfg.position {
            MarqueePosition::Top => h as i32,
            MarqueePosition::Bottom => (logical_h.saturating_sub(h)) as i32,
        };
        let _ = window.set_position(PhysicalPosition::new(x, y));
    }
}

pub fn show(app_handle: &tauri::AppHandle, title: &str, body: &str, marquee_cfg: &MarqueeConfig) {
    let text = format!("{} — {}", title, body);

    // Collect all window labels that start with "marquee"
    let mut labels: Vec<String> = Vec::new();
    for label in app_handle.webview_windows().keys() {
        if label == "marquee" || label.starts_with("marquee-") {
            labels.push(label.clone());
        }
    }

    for label in &labels {
        if let Some(window) = app_handle.get_webview_window(label) {
            reposition_marquee(&window, marquee_cfg);
            let _ = window.emit("marquee-text", &text);
            let _ = window.emit("marquee-config", marquee_cfg);
            let _ = window.show();
            let _ = window.set_ignore_cursor_events(true);
        }
    }

    // Auto-hide after configured duration
    let duration = marquee_cfg.duration_secs as u64;
    let handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(duration)).await;
        for label in &labels {
            if let Some(w) = handle.get_webview_window(label) {
                let _ = w.hide();
            }
        }
    });
}
