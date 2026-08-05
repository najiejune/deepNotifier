/// Update the tray tooltip to reflect the latest notification.
pub fn update_tooltip(app_handle: &tauri::AppHandle, title: &str) {
    if let Some(tray) = app_handle.tray_by_id("main-tray") {
        let _ = tray.set_tooltip(Some(format!("deepNotifier: {}", title)));
    }
}
