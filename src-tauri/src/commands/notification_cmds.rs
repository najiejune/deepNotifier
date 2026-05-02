use crate::notifier::dispatcher::NotificationEvent;
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
