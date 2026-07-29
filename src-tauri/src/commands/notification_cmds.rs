use crate::notifier::dispatcher::NotificationEvent;
use crate::notifier::window_focus::{self, FocusDiagnostic};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_notifications(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<NotificationEvent>, String> {
    let history = state.history.read().await;
    let items: Vec<NotificationEvent> = history
        .iter()
        .take(limit.unwrap_or(100))
        .cloned()
        .collect();
    Ok(items)
}

#[tauri::command]
pub async fn clear_notifications(state: State<'_, AppState>) -> Result<(), String> {
    let mut history = state.history.write().await;
    history.clear();
    Ok(())
}

/// Called when the user clicks a system notification toast.
/// Focuses the source process window if we stored a PID for the last notification.
#[tauri::command]
pub async fn focus_pending_pid(state: State<'_, AppState>) -> Result<(), String> {
    let pid = {
        let mut guard = state.pending_pid.write().await;
        guard.take()
    };
    tracing::info!(?pid, "focus_pending_pid called");
    if let Some(pid) = pid {
        window_focus::bring_pid_to_front(pid);
    } else {
        tracing::warn!("focus_pending_pid: pending_pid is None — no PID was stored");
    }
    Ok(())
}

// ── Debug / test commands ──────────────────────────────────────────

/// Test command: try to focus a window for the given PID and return diagnostics.
#[tauri::command]
pub async fn debug_focus_pid(pid: u32) -> Result<FocusDiagnostic, String> {
    tracing::info!(pid, "debug_focus_pid called");
    let diag = window_focus::diagnose_focus(pid);
    Ok(diag)
}

/// Test command: read the current pending_pid without consuming it.
#[tauri::command]
pub async fn debug_get_pending_pid(state: State<'_, AppState>) -> Result<Option<u32>, String> {
    let guard = state.pending_pid.read().await;
    let pid = *guard;
    tracing::info!(?pid, "debug_get_pending_pid");
    Ok(pid)
}
