use tauri_plugin_notification::NotificationExt;

pub fn notify(app_handle: &tauri::AppHandle, title: &str, body: &str) {
    // Update tray tooltip
    if let Some(tray) = app_handle.tray_by_id("main-tray") {
        let _ = tray.set_tooltip(Some(format!("deepNotifier: {}", title)));
    }

    // Send native OS notification (Windows Toast / macOS / Linux)
    if let Err(e) = app_handle
        .notification()
        .builder()
        .title(title)
        .body(body)
        .show()
    {
        tracing::warn!("Failed to send system notification: {}", e);
    }
}
