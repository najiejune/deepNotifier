use std::sync::Mutex;

/// Global PID storage shared between the dispatcher and the toast on_activated callback.
/// Using a plain std Mutex because the toast callback runs on a WinRT thread, not tokio.
pub static NOTIFICATION_CLICK_PID: Mutex<Option<u32>> = Mutex::new(None);

/// Update the tray tooltip to reflect the latest notification.
pub fn update_tooltip(app_handle: &tauri::AppHandle, title: &str) {
    if let Some(tray) = app_handle.tray_by_id("main-tray") {
        let _ = tray.set_tooltip(Some(format!("deepNotifier: {}", title)));
    }
}

/// Send a native OS toast notification.
///
/// On Windows this uses `tauri-winrt-notification` directly so we can register an
/// `.on_activated()` handler that focuses the source CLI window on click.
/// On other platforms we fall back to a fire-and-forget toast (click-to-focus is
/// handled by the frontend via `window.Notification.onclick`).
pub fn notify(title: &str, body: &str) {
    #[cfg(target_os = "windows")]
    notify_windows(title, body);

    #[cfg(not(target_os = "windows"))]
    notify_other(title, body);
}

#[cfg(target_os = "windows")]
fn notify_windows(title: &str, body: &str) {
    let pid = *NOTIFICATION_CLICK_PID.lock().unwrap();

    // Use POWERSHELL_APP_ID so toasts work without a registered AppUserModelID
    // (same approach as notify-rust in dev mode).
    let toast = tauri_winrt_notification::Toast::new(
        tauri_winrt_notification::Toast::POWERSHELL_APP_ID,
    )
    .title(title)
    .text1(body);

    let toast = if let Some(pid) = pid {
        toast.on_activated(move |_args| {
            tracing::info!(pid, "toast on_activated → focusing window");
            crate::notifier::window_focus::bring_pid_to_front(pid);
            Ok(())
        })
    } else {
        toast
    };

    if let Err(e) = toast.show() {
        tracing::warn!("Failed to show Windows toast: {}", e);
    }
}

#[cfg(not(target_os = "windows"))]
fn notify_other(title: &str, body: &str) {
    // On macOS / Linux we rely on the frontend window.Notification for click handling.
    // Just show a system toast via notify-rust as a fallback.
    if let Err(e) = notify_rust::Notification::new()
        .summary(title)
        .body(body)
        .show()
    {
        tracing::warn!("Failed to show system notification: {}", e);
    }
}
